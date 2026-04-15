# Exploration: Content-Addressed Hashing Across the Benten Platform

**Created:** 2026-04-11
**Purpose:** Deep exploration of content-addressed hashing as a fundamental architectural element -- alongside the 12 operation primitives, structural invariants, and the self-evaluating graph model. This explores how content hashing applies to every layer: module identity, data integrity, sync verification, version identity, deduplication, governance, trust attestation, Merkle trees for efficient sync, subgraph snapshots, and content-addressed storage.
**Status:** Research exploration (pre-design)
**Dependencies:** `SPECIFICATION.md`, `operation-vocab-p2p.md` (Section 11), `critique-p2p.md`, `critique-holochain-perspective.md`, `critique-crdt-graph.md`, `critique-data-integrity.md`, `critique-mesh-sync.md`

---

## 1. The Core Idea: Two Kinds of Identity

Every system that uses content-addressed hashing faces a fundamental tension between two kinds of identity:

**Anchor identity** (stable, mutable): "This is the blog post called 'Hello World'. It has been edited 47 times. Its ID is `post-7f3a`."

**Content identity** (unstable, immutable): "This is the exact blob of bytes whose SHA-256 hash is `abc123`. If any byte changes, it is a different blob with a different hash."

The Benten engine already has this duality built into its version chain model: **anchors provide stable identity; version Nodes capture state at a point in time**. The insight of this exploration is that content-addressed hashing can unify and strengthen this duality by giving every version Node a cryptographic content hash, and then extending the same principle to every layer of the platform.

### 1.1 The Dual Identity Model

```
Anchor (stable identity)
  |
  |-- CURRENT --> Version v3 (content hash: 7f3a2b...)
  |                  |-- NEXT_VERSION <-- Version v2 (content hash: e91d44...)
  |                                          |-- NEXT_VERSION <-- Version v1 (content hash: b0c8f1...)
```

The anchor is how the world refers to this entity: "post-7f3a" or "module:seo-enrichment". The content hash is how the engine verifies this entity: "the exact state I received matches the hash I was told to expect."

Neither replaces the other:
- You cannot use content hashes as stable references because they change on every edit.
- You cannot use anchor IDs for integrity verification because they say nothing about content.
- Together, they provide both stability (anchors) and verifiability (hashes).

### 1.2 What Gets Hashed

The content hash of a version Node is computed from:

1. **The Node's labels** (sorted alphabetically for determinism)
2. **The Node's properties** (keys sorted, values serialized canonically -- see Section 12 on canonicalization)
3. **NOT** the Node's anchor ID (the same content under two different anchors produces the same hash)
4. **NOT** the Node's metadata timestamps (HLC, creation time -- these are instance-specific)
5. **NOT** the Node's edges (edges are hashed separately; see Section 5 on Merkle subgraphs)

This means: **the content hash identifies WHAT the data is, not WHO created it, WHEN, or WHERE it lives.** Two instances that independently create a Node with identical labels and properties produce identical content hashes. This is the foundation of deduplication and verification.

---

## 2. How Existing Systems Use Content-Addressed Hashing

### 2.1 IPFS: Content-Addressed Storage

**How it works:** IPFS stores data as blocks identified by Content Identifiers (CIDs). A CID encodes: the hash algorithm used (typically SHA-256), the codec for interpreting the data (dag-pb for files, dag-cbor for structured data), and the hash digest itself. Files are chunked into blocks, each block gets a CID, and a Merkle DAG links the blocks. The root CID identifies the entire file.

**What works:**
- Global deduplication: the same file uploaded by two different users is stored once.
- Integrity verification: anyone with a CID can verify the data matches.
- Location-independent addressing: request data by CID, any node that has it can serve it.
- Immutable references: a CID always refers to the exact same data. Links never go stale.

**What does not work:**
- **Mutability is bolted on.** IPNS (InterPlanetary Name System) maps a mutable name to a CID, but it adds latency (DNS-like resolution), complexity, and an entirely separate naming layer. The fundamental data model is immutable; mutability is an afterthought.
- **Garbage collection is hard.** If nobody pins a CID, the data eventually disappears. There is no concept of "this CID is referenced by these other CIDs, so keep it alive." Reference counting in a distributed system is an unsolved problem.
- **Large file performance.** Chunking large files into thousands of blocks creates deep Merkle DAGs. Fetching the file requires traversing the DAG, which means many round trips. IPFS mitigates this with bitswap (parallel block fetching), but the fundamental overhead remains.
- **No query capability.** You can fetch data by CID, but you cannot query across CIDs. "Find all blog posts with tag X" is not expressible. IPFS is a storage layer, not a database.

**Lessons for Benten:**
- Content addressing for data blocks (version Nodes) is proven at planetary scale.
- The mutable/immutable duality (anchor + content hash) is the right design. IPFS's struggles with IPNS validate that mutability must be a first-class concept, not an afterthought.
- Benten should NOT try to be IPFS. The engine is a graph database with IVM, not a content-addressed block store. But it should use the same hashing primitives for integrity verification and deduplication.
- The CID format (multicodec + multihash) is worth adopting for future-proofing: if Benten ever needs to change hash algorithms, the hash itself declares what algorithm produced it.

### 2.2 Git: Merkle Trees for Code History

**How it works:** Git stores four types of objects, each identified by its SHA-1 (migrating to SHA-256) hash:
- **Blob:** raw file content
- **Tree:** directory listing (pointers to blobs and other trees)
- **Commit:** snapshot of a tree + parent commit(s) + author + message
- **Tag:** named pointer to a commit

A commit's hash depends on its tree hash, which depends on the hashes of all blobs and subtrees. Change one file, and every hash up the chain to the root commit changes.

**What works:**
- **Efficient sync.** `git fetch` compares local and remote commit hashes. It only transfers objects the local side does not have. For a repository with 1 million objects where 100 changed, Git transfers ~100 objects, not 1 million.
- **Integrity verification.** Clone a repository, compute the hash of every object, compare against the stored hashes. If any mismatch, the repository is corrupted. This is O(n) where n is the number of objects, but it is embarrassingly parallel.
- **Deduplication within a repository.** Two files with identical content share the same blob object. Two commits that produce identical directory trees share the same tree object.
- **Pack files.** Git compresses similar objects together using delta encoding. The pack algorithm exploits content similarity (not just identical content) for compression ratios that pure deduplication cannot achieve.
- **Partial clone and sparse checkout (Git 2.25+).** A client can request a subset of the repository based on path patterns. The server filters the packfile. This enables working with massive repositories without downloading everything.

**What does not work:**
- **No content-addressed references between repositories.** Git objects are local to a repository. You cannot reference a blob in another repository by its hash. Git submodules and subtrees are bolt-on solutions that do not use content addressing.
- **SHA-1 collision vulnerability.** The SHAttered attack (2017) produced two different PDF files with the same SHA-1 hash. Git's migration to SHA-256 addresses this but is not yet complete (2026). This is a cautionary tale: hash algorithm choice matters, and migration is painful.
- **No semantic understanding of content.** Git hashes bytes, not structure. Two JSON files that differ only in whitespace have different hashes, even though they represent the same data. Canonical serialization is essential for structured data.

**Lessons for Benten:**
- Merkle trees are the proven mechanism for efficient sync. Benten's subgraph sync should use the same principle: compare root hashes, drill into subtrees that differ, transfer only the differences.
- Pack-file-style delta compression should be considered for large sync transfers.
- **Canonical serialization is mandatory.** Benten version Nodes contain structured data (properties as key-value pairs). The same logical data must always produce the same hash. This requires a canonical serialization format (see Section 12).
- **Hash algorithm choice matters.** Use SHA-256 (not SHA-1). Use a self-describing format (multihash) so the algorithm can be upgraded without breaking all existing hashes.

### 2.3 AT Protocol: Merkle Search Trees for User Data Repositories

**How it works:** In the AT Protocol (Bluesky), each user's data lives in a Personal Data Repository (repo). The repo is structured as:
- A **signed commit** pointing to the root of a Merkle Search Tree (MST)
- The MST maps collection/record-key paths to record CIDs
- Each record is a CBOR-encoded data blob identified by its CID
- The commit is signed by the user's key pair, making the entire repo state cryptographically attributable

The MST is a probabilistic balanced search tree where the tree structure is determined by the hashes of the keys. This means two repos with the same set of records produce the same MST structure, regardless of insertion order. The MST is structurally deterministic.

**What works:**
- **Efficient diff between repo states.** To sync, compare the MST root CIDs. If they match, repos are identical. If they differ, walk down the tree comparing node CIDs until you find the differing records. Transfer only those records. This is O(log n) comparisons for a single change in a repo of n records.
- **Portable data.** Because the repo is a self-contained, signed Merkle structure, it can be exported as a CAR (Content ARchive) file and imported to another PDS. Your data is not locked to a server.
- **Cryptographic attribution.** The signed commit proves the repo owner authorized the state. No relay or server can forge data in your repo.
- **Collection-scoped sync (2026).** Consumers can subscribe to specific collections rather than the entire repo. The MST structure enables efficient extraction of subtrees.

**What does not work:**
- **Single-writer assumption.** Each repo has exactly one writer (the account owner). Multi-writer collaboration is not supported at the repo level. This simplifies everything but limits the use cases (no real-time collaborative editing within a repo).
- **No graph structure.** The MST is a flat key-value mapping (collection + record-key -> CID). There are no edges, no relationships, no traversal. Cross-record references are application-level strings, not first-class primitives. Benten's graph model is fundamentally more expressive.
- **Schema rigidity.** Record schemas are defined by Lexicon files and are effectively immutable per collection version. Schema evolution requires creating a new collection version. This is safe but inflexible.

**Lessons for Benten:**
- The MST's deterministic structure (same data = same tree, regardless of insertion order) is a property Benten's Merkle subgraph hashing should aim for. This requires canonical ordering of Nodes and Edges within a subgraph hash computation.
- The signed commit model is directly applicable to Benten's version chains: each version Node should carry a signature from the authoring instance, proving the mutation was authorized.
- **CAR-file-equivalent export.** Benten should support exporting a subgraph as a self-contained, signed, content-addressed archive. This enables data portability and offline verification.
- The AT Protocol's success with MST-based sync validates that Merkle tree diffing is production-ready for user data repositories.

### 2.4 Holochain: Content-Addressed Entries with Agent Source Chains

**How it works:** In Holochain, every piece of data (an "Entry") is identified by the hash of its content. When an agent creates an Entry:
1. The Entry is hashed (its content becomes its address)
2. An Action record is created on the agent's source chain (an append-only, hash-linked log)
3. The Action references the Entry hash
4. If the Entry is public, it is published to the DHT at the address determined by its hash

The source chain is itself a hash chain: each Action contains the hash of the previous Action. This makes the chain tamper-evident.

**What works:**
- **Automatic deduplication.** Two agents who independently create identical Entries produce the same hash. The DHT stores the Entry once.
- **Integrity verification.** Any peer can verify an Entry by recomputing its hash. If the computed hash does not match the address, the Entry is corrupt or forged.
- **Tamper-evident history.** The hash-linked source chain means any tampering with past Actions breaks the chain. Validators can detect this.
- **Content-addressed validation.** Validation rules can reference other Entries by hash. The validator can fetch and verify the referenced Entry as part of validation. This enables cross-Entry integrity constraints.

**What does not work:**
- **No stable identity.** Because Entry identity IS the content hash, updating an Entry creates a new hash (new identity). Holochain works around this with "original entry hash" patterns, but it is a convention, not a primitive. Every update requires following a chain of update Actions to find the latest version. This is exactly the problem Benten's anchors solve.
- **Tombstone accumulation.** Deleting an Entry creates a Delete Action on the source chain, but the original Entry still exists on the DHT until all peers garbage-collect it. Tombstones accumulate indefinitely in the current implementation.
- **Performance of hash computation.** Hashing every Entry on every create/update is a real cost. For small Entries (kilobytes), this is negligible. For large Entries (megabytes of binary data), it can be noticeable. Holochain mitigates this by splitting large data across multiple Entries.

**Lessons for Benten:**
- The source chain concept maps directly to Benten's version chains with content hashes: each version Node's hash serves the same role as a Holochain Action's hash.
- **Content-addressed validation is powerful.** Benten's validation rules should be able to reference other Nodes by content hash, not just by anchor ID. This enables "verify that this Node references a specific version of that Node" -- a stronger integrity guarantee than referencing the anchor (which might have been updated since the reference was created).
- The "no stable identity" problem validates Benten's anchor + content hash dual identity model.
- Holochain's 8 years of experience proves that content-addressed data is viable for real applications. The challenges (garbage collection, performance, stable identity) are all solvable.

### 2.5 Unison: Content-Addressed Code

**How it works:** In the Unison programming language, every definition (function, type, value) is identified by the hash of its Abstract Syntax Tree (AST). The hash is computed after all names are resolved to their own hashes. This means:
- Renaming a function does not change its hash (the hash is structural, not nominal)
- Two developers who independently write the same function produce the same hash
- Importing a function means recording its hash, not its name
- A function's hash changes if and only if its behavior changes

**What works:**
- **Fearless refactoring.** Renaming does not break anything because nothing refers to names -- everything refers to hashes.
- **Guaranteed reproducibility.** A function identified by hash X always does exactly the same thing. There are no dependency version conflicts.
- **Efficient code sharing.** When sharing code between instances, only hashes that the receiver does not have need to be transferred.
- **Natural caching.** If you have already computed the result of function hash X with input hash Y, the result is cached forever (the function cannot change, so the result cannot change).

**What does not work:**
- **Discoverability.** Content hashes are opaque. You cannot look at a hash and know what it does. Names are essential for human understanding. Unison solves this with a "codebase" that maps names to hashes, but this creates a separate mutable naming layer.
- **Schema evolution.** If a type changes, its hash changes, which changes the hash of every function that uses the type. This cascading hash invalidation can require updating large swaths of the codebase for a minor type change.
- **Debugging.** Stack traces reference hashes, not names. Tooling must translate hashes back to human-readable names.

**Lessons for Benten:**
- Unison validates the exact model proposed in `operation-vocab-p2p.md` Section 11: operation subgraphs identified by content hash, independent of naming. Benten's operation Nodes should follow this model.
- **The cascading hash invalidation problem is real.** If a content type definition's hash changes (because a field was added), the hash of every operation subgraph that references that content type also changes. Benten should address this by having operation subgraphs reference content types by anchor ID (stable), not by content hash (unstable). The content hash is for verification and deduplication, not for reference.
- Names are for humans; hashes are for machines. Benten needs both: human-readable module names for discoverability, content hashes for integrity verification.

### 2.6 Package Managers: Hash-Based Integrity Verification

**npm:** The `package-lock.json` file records an `integrity` field for every dependency: a SHA-512 hash of the .tgz package tarball. On install, npm downloads the package, recomputes the hash, and rejects it if the hash does not match. This prevents supply-chain attacks where a package is modified after publication.

**Cargo (Rust):** The `Cargo.lock` file records the `checksum` for every crate. Crates.io publishes checksums alongside crates. `cargo install` verifies checksums before compiling.

**Nix:** The Nix package manager takes this further with content-addressed derivations. Every package's store path is derived from the hash of its build inputs (source code, compiler flags, dependencies, build script). Two machines that build the same package with the same inputs produce the same store path. This is the most aggressive use of content addressing in package management.

**Lessons for Benten:**
- Module integrity verification should follow the npm/Cargo model: every module has a published hash, and installation verifies the hash.
- Nix's content-addressed derivations suggest that Benten modules could be identified by the hash of their definitions (schema + validation rules), similar to how Holochain's DNA hash works. Two instances running a module with the same definition hash are provably running the same module.

---

## 3. Module Identity and Attestation

### 3.1 The Module Hash

A Benten module is, in the engine's model, a subgraph of definition Nodes (content types, blocks, field types, routes, event handlers) connected by structural Edges. The module's content hash is computed from this subgraph:

```
Module Hash = hash(
  sorted(definition_node_hashes) +
  sorted(edge_hashes_between_definitions) +
  sorted(operation_subgraph_hashes)
)
```

Where each definition Node's hash is computed from its content (labels + properties, canonically serialized), and each operation subgraph's hash follows the model in `operation-vocab-p2p.md` Section 11.

**What the module hash tells you:**
- Two instances running a module with the same hash have provably identical definitions and handler logic.
- If a module is updated (new field added to a content type, handler logic changed), the hash changes.
- The hash does NOT include runtime state (settings values, cached data). It identifies the module's DEFINITION, not its current state.

### 3.2 Separating Definition Hash from Configuration Hash

Following the Holochain DNA model, a module's identity has two layers:

| Layer | What It Hashes | When It Changes | Analogy |
|---|---|---|---|
| **Definition hash** | Schema definitions, validation rules, operation subgraphs | When the module is updated (new version) | Holochain DNA hash |
| **Configuration hash** | Module settings values, operator-configured options | When the operator changes settings | Holochain DNA properties |

Two instances with the same definition hash but different configuration hashes are running the same module code with different settings. They can sync data (because they agree on schema) but may behave differently (because settings differ).

Two instances with different definition hashes are running different module versions. Sync for content types affected by the definition change should be paused until the mismatch is resolved.

### 3.3 Community Attestation via Hash

When a community (a Digital Garden or Grove) trusts a module, they attest to its hash:

```
[Garden: "Photography Club"]
  |-- ATTESTS_MODULE --> [Attestation Node]
                            |-- targetHash: "sha256-7f3a2b..."
                            |-- moduleAnchor: "module:photo-gallery"
                            |-- attestedAt: 2026-04-11
                            |-- attestedBy: [DID of Garden admin]
                            |-- signature: [Ed25519 signature]
```

This Attestation Node is itself part of the graph. It syncs like any other data. When a member of the Photography Club sees a recommendation to install the "photo-gallery" module, they can verify:
1. The Photography Club has attested hash `7f3a2b...`
2. The module they are about to install has hash `7f3a2b...`
3. These match -- the module is what the Photography Club reviewed

### 3.4 Transitive Trust Chains

Trust can be transitive. If the Photography Club trusts the "Photography Tools" collective, and the collective has attested hash `7f3a2b...`, then club members who trust the club implicitly trust the collective's attestation.

```
[Alice's Instance]
  |-- TRUSTS --> [Photography Club]
                    |-- TRUSTS --> [Photography Tools Collective]
                                      |-- ATTESTS_MODULE --> hash: 7f3a2b...
```

The trust chain is a graph traversal. Alice's instance can compute: "This module hash is attested by an entity I transitively trust." The depth of the chain affects confidence (direct trust > one-hop trust > two-hop trust).

**Limits on transitivity:** The engine should enforce a maximum trust chain depth (default: 3 hops). Beyond that, the attenuation of trust makes the attestation meaningless. This mirrors UCAN's delegation depth limits.

### 3.5 Revocation of Attestations

If a module is found to be malicious after attestation:
1. The attesting entity creates a Revocation Node referencing the Attestation Node
2. The Revocation supersedes the Attestation (remove-wins for security edges, per `critique-crdt-graph.md` Scenario 5)
3. The Revocation syncs to all members via the normal sync mechanism
4. Members' instances check for revocations before trusting an attestation

Revocations should have a TTL (time-to-live) on the attestation itself. An attestation that says "valid for 90 days" automatically expires, forcing periodic re-attestation. This bounds the damage from delayed revocation propagation.

---

## 4. Data Integrity

### 4.1 Hash-Verified Version Chains

Each version Node in a version chain carries a content hash. This enables:

**Tamper detection:** When syncing a version chain, the receiver recomputes the hash of each received version Node. If the computed hash does not match the claimed hash, the version has been tampered with.

**Complete chain verification:** Walk the version chain from CURRENT back to v1. At each step, verify: (a) the version Node's content hash is correct, (b) the NEXT_VERSION edge correctly links to the previous version. If any check fails, the chain is corrupt.

**Selective verification:** To verify just the current state (without walking the full history), verify only the CURRENT pointer's target version Node. This is O(1) -- one hash computation and comparison.

### 4.2 Signed Versions

Content hashes prove "this data has not been modified." They do not prove "this data was created by a particular author." For that, version Nodes should also carry a signature:

```
Version Node {
  labels: ["ContentType:post"],
  properties: { title: "Hello", body: "World", ... },
  contentHash: "sha256-7f3a2b...",
  authorDID: "did:key:z6Mk...",
  signature: "ed25519-[signature of contentHash by author's key]"
}
```

The signature proves: the entity identified by `authorDID` authored this specific content (identified by `contentHash`) at this specific version. Combined with UCAN capability chains, this proves the author was authorized to make this change.

### 4.3 Hash-Based Conflict Detection During Sync

When two instances sync a shared Node:

1. Compare the CURRENT version's content hash
2. If hashes match: the instances have the same current state. No sync needed for this Node.
3. If hashes differ: the instances have diverged. Proceed to CRDT merge.

This "compare hashes first" step is the fast path that avoids transferring data for Nodes that have not changed. In a subgraph of 10,000 Nodes where 50 have changed, this reduces the comparison from 10,000 full content comparisons to 10,000 hash comparisons (trivial) + 50 content transfers.

### 4.4 Hash-Based Dataset Verification

A dataset (a knowledge graph, a content collection, a CMS export) can be verified by computing a single root hash:

1. Hash each Node in the dataset
2. Build a Merkle tree over the sorted Node hashes
3. The root hash represents the entire dataset

Anyone who receives the dataset and the root hash can verify that every Node is present and unmodified. This is particularly valuable for:
- **Regulatory compliance:** "The data I exported on April 11, 2026 has root hash X. Here is the signed attestation."
- **Academic citations:** "This analysis was performed on dataset with root hash X."
- **Legal proceedings:** "The evidence has not been modified since collection. Here is the root hash from the original collection date."

---

## 5. Merkle Trees for Efficient Sync

### 5.1 The Merkle Subgraph

A subgraph in Benten is defined by a traversal pattern (roots + edge types + max depth). The Merkle tree over a subgraph is constructed as follows:

```
Level 0 (leaves): Hash of each Node in the subgraph
Level 1: Hash of pairs of Level 0 hashes
Level 2: Hash of pairs of Level 1 hashes
...
Root: Single hash representing the entire subgraph
```

The construction must be deterministic: same Nodes in same order produce same tree. This requires a canonical ordering of Nodes within the subgraph (e.g., sorted by anchor ID, or by content hash).

### 5.2 The Sync Protocol Using Merkle Trees

When Instance A syncs subgraph S with Instance B:

```
Step 1: A computes the Merkle root of S.
         B computes the Merkle root of S.
         They exchange roots.

Step 2: If roots match -> S is identical on both instances. Done.

Step 3: If roots differ -> exchange Level 1 hashes.
         Compare Level 1 hashes to identify which subtrees differ.

Step 4: For differing subtrees, exchange Level 2 hashes.
         Continue drilling until individual differing Nodes are identified.

Step 5: Exchange only the differing Nodes (and their version chains).

Step 6: Apply CRDT merge rules to reconcile differences.
```

**Complexity analysis:**
- For a subgraph of N Nodes where K have changed:
  - Hash comparisons: O(K * log(N))
  - Data transferred: O(K) Nodes
- Compare to naive sync (transfer all Nodes): O(N) data transferred
- For N=10,000 and K=50: Merkle sync transfers ~50 Nodes with ~300 hash comparisons. Naive sync transfers 10,000 Nodes.

### 5.3 Incremental Merkle Tree Maintenance

Computing a Merkle tree from scratch over 10,000 Nodes on every sync is expensive. The engine should maintain the tree incrementally:

1. When a Node is created/updated/deleted, recompute its hash
2. Walk up the Merkle tree, recomputing parent hashes until the root
3. This is O(log N) per mutation -- the same cost as a B-tree index update

The IVM system is the natural place for this: the Merkle tree IS a materialized view over the subgraph. When the subgraph changes, the Merkle tree updates incrementally, just like any other materialized view.

### 5.4 Merkle Trees and Subgraph Boundaries

A critical question: how do Merkle trees interact with dynamic subgraph boundaries?

If the subgraph boundary is traverse-based (Section 3 of `critique-mesh-sync.md`), adding a new Edge can bring new Nodes into scope. This changes the Merkle tree even though no existing Node was modified. The Merkle tree must be re-evaluated whenever the boundary is re-evaluated.

**Recommendation:** Maintain a materialized membership set for each sync agreement (as recommended by `critique-mesh-sync.md`). The Merkle tree is built over this membership set. When the boundary is re-evaluated and new Nodes enter scope, they are added to the membership set, and the Merkle tree is updated incrementally.

### 5.5 Relationship to Holochain's Quantised Gossip

Holochain's Kitsune2 uses a similar principle but adapted for DHT shards: divide the address space into regions, compare region hashes, drill into differing regions. The difference is that Kitsune2 uses a 2D grid (time x address space), while Benten's Merkle trees use a 1D tree (sorted by Node identity).

For Benten's use case (selective subgraph sync between known peers), a 1D Merkle tree is simpler and sufficient. The 2D grid is optimized for DHT gossip across many anonymous peers, which is not Benten's model.

### 5.6 Relationship to AT Protocol's MST

AT Protocol's Merkle Search Tree is a more sophisticated variant: a probabilistic balanced search tree where the tree structure is determined by key hashes. This gives it the property of structural determinism: two instances with the same records produce the same tree structure regardless of insertion order.

Benten should aim for the same property. The simplest way: sort the subgraph's Nodes by anchor ID (which is stable across instances), then build a standard binary Merkle tree. The anchor ID ordering ensures structural determinism.

---

## 6. Version Identity

### 6.1 Content Hash as Version Fingerprint

Each version Node in a version chain has a content hash. This hash serves as the version's fingerprint:

```
Two instances can quickly compare "do we have the same version?"
  Instance A: CURRENT -> version with hash abc123
  Instance B: CURRENT -> version with hash abc123
  -> Match. Same version. No sync needed.
```

This is faster than comparing full content: a hash comparison is a 32-byte equality check.

### 6.2 Version Comparison Without Data Transfer

When two instances want to determine their sync delta for a specific Node:

1. Exchange the CURRENT version's content hash -> quickly determines if current state matches
2. If different, exchange the hash of each version in the chain -> identifies the divergence point (last common version)
3. Transfer only versions newer than the divergence point

For a Node with 100 versions where the last 5 diverge: exchange 100 hashes (3.2KB total for SHA-256), identify the divergence at version 95, transfer 5 version Nodes. Without hashes: must transfer all 100 versions and compare content.

### 6.3 Hash Chain Integrity

The version chain itself can be verified as a hash chain (similar to a blockchain or Holochain source chain):

```
Version v3:
  contentHash: sha256-abc123
  parentHash: sha256-def456    (hash of v2)
  signature: ed25519-[sig]

Version v2:
  contentHash: sha256-def456
  parentHash: sha256-ghi789    (hash of v1)
  signature: ed25519-[sig]

Version v1:
  contentHash: sha256-ghi789
  parentHash: null              (genesis version)
  signature: ed25519-[sig]
```

Each version references its parent's hash. This creates a tamper-evident chain: modifying any historical version changes its hash, which invalidates all subsequent versions' parentHash references. An attacker cannot alter history without breaking the chain.

### 6.4 Version DAG, Not Version Chain

As `critique-crdt-graph.md` (Scenario 3) recommends, concurrent edits produce a version DAG, not a linear chain. Content hashes strengthen this model:

```
v1 (hash: aaa)
  |-- v2a (hash: bbb, parents: [aaa])  -- Instance A's edit
  |-- v2b (hash: ccc, parents: [aaa])  -- Instance B's edit
  |-- v3 (hash: ddd, parents: [bbb, ccc])  -- Merged version
```

The merge version v3 references both parents by hash. Any instance can verify that v3 was produced by merging exactly v2a and v2b. This is the exact model Git uses for merge commits.

---

## 7. Sync Verification

### 7.1 Verifying Sync Integrity

When Instance B receives data from Instance A during sync:

1. For each received version Node: recompute content hash, verify it matches the claimed hash
2. For each version chain: verify parentHash links are consistent
3. For each signed version: verify signature against the author's public key
4. For the complete sync batch: verify the Merkle root of all received Nodes matches the Merkle root A claimed

If any verification fails, reject the entire sync batch. Log a violation report (per `critique-holochain-perspective.md` Section 2b) attributing the invalid data to Instance A.

### 7.2 Man-in-the-Middle Protection

Even if the transport is compromised (an attacker intercepts and modifies data in transit), content hashes detect the tampering. The receiver recomputes hashes and discovers mismatches. Combined with signatures, the receiver can also verify the data was actually authored by the claimed author.

This is defense-in-depth: TLS protects the transport, content hashes protect the data, signatures protect attribution. Any one layer can fail and the remaining layers still provide protection.

### 7.3 Partial Sync Verification

For large sync transfers, the receiver does not need to wait for the complete transfer before starting verification:

1. Receive the Merkle tree structure first (compact: O(N) hashes for N Nodes)
2. Start receiving individual Nodes
3. Verify each Node as it arrives (hash check)
4. After all Nodes received, verify the Merkle tree is consistent

This enables streaming verification -- a pipeline of receive, hash, verify, apply. Failed verification at any step halts the pipeline without wasting bandwidth on subsequent data.

---

## 8. Deduplication

### 8.1 Intra-Instance Deduplication

If two Nodes on the same instance have identical content (same labels and properties), they have the same content hash. The engine could store the content once and have both Nodes reference the shared storage.

**When this matters:** Templates and copies. A CMS might have 100 pages that started from the same template. If only 10 have been modified, 90 still have identical content. With content-addressed storage, those 90 pages share one stored blob.

**When this does not matter:** Most Node content is unique (different title, different body, different metadata). Deduplication of individual Nodes has diminishing returns in a CMS context.

**Recommendation:** Implement content-addressed storage for binary data (files, images) where deduplication is high-value. For Node properties (small, mostly unique), deduplication overhead exceeds the benefit.

### 8.2 Cross-Instance Deduplication

More valuable: when two instances sync, content hashes identify data they already share:

1. Instance A sends a list of content hashes for the Nodes it wants to sync
2. Instance B responds: "I already have these hashes: [list]. Send me the rest."
3. A sends only the Nodes whose hashes B does not have

This is exactly how Git's fetch protocol works. For a subgraph where both instances have 90% overlap, this reduces data transfer by 90%.

### 8.3 Cross-Garden Deduplication

If the same content (a popular article, a widely-used block template) exists in multiple Digital Gardens, content hashing enables recognition:

```
Garden A has: Block "hero-banner" with hash xyz789
Garden B has: Block "hero-banner" with hash xyz789
  -> Same content. If a member of both Gardens syncs, the engine stores one copy.
```

This is particularly powerful for module definitions: a popular module installed in 1,000 Gardens has 1,000 identical definition subgraphs. Content addressing means the definition is stored once per instance (not per Garden-instance pair).

---

## 9. Governance

### 9.1 Hash-Referenced Governance Proposals

In a Grove (governed community), governance actions reference specific hashes:

```
[Proposal: "Upgrade photo-gallery module"]
  |-- currentModuleHash: sha256-7f3a2b...
  |-- proposedModuleHash: sha256-c4d5e6...
  |-- proposedBy: did:key:z6Mk...
  |-- votingPeriod: 7 days
  |-- quorum: 51%
```

The proposal is unambiguous: upgrade from exactly hash X to exactly hash Y. There is no possibility of a bait-and-switch where the proposed module is modified between the vote and the installation. The hash IS the proposal.

### 9.2 Vote Integrity

Each vote references the proposal's hash:

```
[Vote Node]
  |-- proposalHash: sha256-[hash of proposal Node]
  |-- vote: "approve"
  |-- voter: did:key:z6Mk...
  |-- signature: ed25519-[signature]
```

The vote's reference to the proposal hash ensures the vote is for exactly this proposal, not a modified version. Combined with the voter's signature, this creates a tamper-evident voting record.

### 9.3 Governance Execution Verification

After the vote passes and the module is upgraded:

```
[Execution Record Node]
  |-- proposalHash: sha256-[hash of proposal]
  |-- voteResult: { approve: 15, reject: 3, abstain: 2 }
  |-- executedModuleHash: sha256-c4d5e6...
  |-- executedAt: 2026-04-18
  |-- executedBy: did:key:z6Mk... (the operator)
  |-- signature: ed25519-[signature]
```

Any member can verify: the executed module hash matches the proposed module hash. The vote result meets the quorum. The executor was authorized. The entire chain from proposal to execution is hash-linked and signed.

---

## 10. Trust Attestation and Web of Trust

### 10.1 Attestation as Graph Data

Trust attestations are Nodes and Edges in the graph, synced like any other data:

```
[Entity: "Photography Club" (did:key:z6...)]
  |-- ATTESTS -->
      [Attestation: {
        targetType: "module",
        targetHash: "sha256-7f3a2b...",
        claim: "reviewed-safe",
        confidence: "high",
        expiresAt: 2026-07-11,
        evidence: "sha256-[hash of review report]"
      }]
```

### 10.2 The Trust Graph

Trust relationships form a graph that the engine can traverse to answer queries like:

- "Is this module hash attested by anyone I trust?" -> Graph traversal from my trust roots through TRUSTS edges to ATTESTS edges
- "Who has attested this hash?" -> Reverse traversal from the hash to all Attestation Nodes
- "What is the shortest trust path to this attestation?" -> BFS from my identity to the attestation
- "Are there conflicting attestations?" -> Find all attestations for the same hash with different claims

### 10.3 Trust Levels and Attenuation

Trust attenuates with distance. Each TRUSTS edge can carry a trust level:

```
Alice --TRUSTS(level: 0.9)--> Photography Club --TRUSTS(level: 0.7)--> Photography Tools
```

Alice's transitive trust in Photography Tools is 0.9 * 0.7 = 0.63. Below a configurable threshold (e.g., 0.5), the trust chain is considered too weak to act on.

This is a quantitative web of trust. The engine can compute trust scores by traversing the TRUSTS edges and multiplying levels. IVM can maintain a materialized view of "all modules attested by entities I trust above threshold X" -- making the trust query O(1) at read time.

### 10.4 Attestation Conflict Resolution

What if two trusted entities disagree?

```
Photography Club --ATTESTS--> hash: 7f3a2b, claim: "safe"
Security Auditors --ATTESTS--> hash: 7f3a2b, claim: "unsafe"
```

The engine does not automatically resolve this -- it surfaces the conflict to the user/operator. But it CAN provide decision support:
- Trust score of each attestor (from the user's perspective)
- Recency of each attestation
- Whether either attestation has been revoked
- Whether other entities have corroborating attestations

### 10.5 Bootstrapping Trust

The cold-start problem: when you first install Benten, you have no trust relationships. How do you decide which modules to install?

**Option A: Platform defaults.** Benten ships with a set of "first-party" module hashes pre-attested by the Benten project. These are the CMS, commerce, communications modules. The user trusts these by default (they installed Benten, implying trust in the project).

**Option B: Community directories.** Public directories (web pages, not part of the engine) list module hashes with reviews. The user manually imports attestations from directories they trust.

**Option C: Social trust.** When a user joins a Digital Garden, the Garden's attestation set becomes available. The user implicitly trusts modules the Garden has attested. This is the "my community vouches for these" model.

**Recommendation:** All three, in layers. Platform defaults for the base installation. Community directories for discovery. Social trust for everyday module adoption.

---

## 11. Subgraph Snapshots and Forking

### 11.1 Content-Addressed Snapshots

A snapshot of a subgraph at a point in time is a Merkle root hash:

```
Snapshot Node {
  subgraphId: "agreement:alice-school",
  merkleRoot: "sha256-[root hash]",
  nodeCount: 1847,
  createdAt: 2026-04-11T14:30:00Z,
  createdBy: did:key:z6Mk...,
  signature: ed25519-[sig]
}
```

This snapshot proves: "On April 11, 2026 at 14:30, the subgraph shared between Alice and the school contained exactly these 1,847 Nodes with this exact content." Anyone with the Nodes can verify the claim by recomputing the Merkle root.

### 11.2 Forking with Provable Shared History

When Instance B forks from Instance A:

```
Fork Record Node {
  forkedFrom: "instance:alice (did:key:z6Mk...)",
  forkPoint: {
    snapshotHash: "sha256-[Merkle root at fork time]",
    version: [vector clock at fork time]
  },
  forkedAt: 2026-04-11T14:30:00Z,
  forkedBy: did:key:z6Mk...,
  signature: ed25519-[sig]
}
```

The fork record proves:
- Instance B forked from Instance A (signed by B)
- The fork happened at a specific point in the version history (the vector clock)
- The shared state at the fork point had a specific content (the Merkle root)

### 11.3 Rejoining After Fork

If B wants to re-sync with A after a fork:

1. B presents the fork record (including the forkPoint snapshot hash)
2. A verifies: "Yes, my state at that vector clock version had that Merkle root"
3. Both compute the delta between the fork point and their current state
4. They exchange deltas and apply CRDT merge

The fork point's Merkle root ensures both sides agree on the shared starting point. Without it, they would need to compare their entire histories to find the divergence.

### 11.4 Snapshot Archiving

Periodic snapshots can be archived as CAR-file-equivalent exports:

```
[Snapshot Archive: subgraph-2026-04-11.car]
  - Merkle root: sha256-abc123
  - Contains: 1,847 Nodes + 3,204 Edges
  - Signed by: did:key:z6Mk...
  - Verifiable: recompute Merkle root from contents, compare to declared root
```

This enables:
- **Backup verification:** After restoring from backup, verify the Merkle root matches the archived snapshot.
- **Audit trail:** A sequence of signed snapshots proves the state at each point in time.
- **Legal hold:** A signed snapshot is evidence of the data's state at a specific date.

---

## 12. Content-Addressed Storage

### 12.1 The Question: Should Benten Nodes Be Stored by Content Hash?

In a content-addressed storage (CAS) model, the "address" (storage key) of data IS its content hash. You do not choose where to store data; the hash determines the location. Two identical Nodes are stored at the same location (automatic deduplication).

**Arguments for CAS at the storage layer:**
- Automatic deduplication (no duplicate data on disk)
- Any piece of data can be verified by recomputing its hash (corruption detection)
- Enables content-addressable networking: request data by hash from any peer

**Arguments against CAS at the storage layer:**
- **Indirection cost.** Every read requires two lookups: anchor -> content hash -> data. Benten's anchor-based identity adds a layer of indirection that CAS-native systems (IPFS, Holochain) do not have.
- **Update cost.** Every property change produces a new content hash, requiring a new storage location. The old location is not freed until garbage collection. This is more expensive than in-place updates.
- **Query complexity.** CAS is optimized for point lookups (by hash). Range queries, sorted indexes, and property-based filters require secondary indexes that operate independently of the CAS addressing.
- **Not needed for the common case.** Most Benten operations are anchor-based: "get the current version of Node X." The anchor lookup already goes through the CURRENT pointer. Adding a CAS layer underneath does not improve this path.

### 12.2 The Recommended Hybrid Model

Do not use CAS as the primary storage model. Use it selectively:

| Data Type | Storage Model | Rationale |
|---|---|---|
| **Node properties** | Anchor-addressed (primary key = anchor ID) | Fast point lookups, in-place updates, IVM integration |
| **Version Nodes** | Content-addressed (hash stored as metadata, used for verification and sync) | Tamper detection, efficient sync, version comparison |
| **Binary data (files, images)** | Content-addressed (stored by hash, referenced from Nodes) | High deduplication value, integrity verification, lazy loading during sync |
| **Operation subgraphs** | Content-addressed (hash IS the identity, per Unison model) | Deduplication, integrity, cache keying |
| **Module definitions** | Content-addressed (hash for verification) + anchor-addressed (for mutable name) | Attestation by hash, but discoverable by name |

### 12.3 The Binary Data CAS

For files and images, a content-addressed store is clearly the right model:

```
File upload:
  1. Compute SHA-256 hash of file bytes
  2. Store file at path: /store/blobs/sha256/7f/3a/7f3a2b...
  3. Create a File Node in the graph with property contentHash: "sha256-7f3a2b..."
  4. If the file already exists (same hash), skip the write. Increment reference count.

File retrieval:
  1. Read File Node from graph
  2. Get contentHash from properties
  3. Fetch blob from CAS by hash
  4. Verify: recompute hash of fetched bytes, compare to stored hash

File deduplication:
  100 users upload the same profile picture.
  All 100 File Nodes have the same contentHash.
  The blob is stored once.
```

### 12.4 Content-Addressed Networking

In a P2P context, content-addressed storage enables content-addressed networking:

```
Instance A wants a file with hash sha256-7f3a2b...
  1. A broadcasts: "Who has blob sha256-7f3a2b?"
  2. Any instance that has the blob can respond
  3. A downloads the blob from the fastest respondent
  4. A verifies: hash of downloaded bytes == sha256-7f3a2b
  5. The source is irrelevant -- the hash guarantees integrity
```

This is exactly how IPFS and BitTorrent work. For Benten, this is relevant in the "Digital Garden" tier where a community hosts shared content. If a member needs a file that exists on multiple community members' instances, they can fetch it from the nearest/fastest source.

---

## 13. Canonicalization: The Hidden Prerequisite

### 13.1 Why Canonical Serialization Matters

Content hashing requires that the same logical data always produces the same byte sequence, which always produces the same hash. If two representations of the same data produce different byte sequences, they produce different hashes, and the entire system breaks.

**Where non-determinism hides:**
- Object key ordering: `{ "a": 1, "b": 2 }` vs `{ "b": 2, "a": 1 }` -- same data, different bytes in most serialization formats
- Floating point representation: `1.0` vs `1.00` vs `1` -- same value, potentially different bytes
- String encoding: NFC vs NFD Unicode normalization
- Whitespace in serialized formats (JSON, CBOR with indefinite-length encoding)
- Null vs absent vs undefined in property maps

### 13.2 The Canonical Serialization for Benten

**Recommended format: CBOR with deterministic encoding (RFC 8949, Section 4.2)**

CBOR (Concise Binary Object Representation) is a binary format that:
- Has a well-defined deterministic encoding profile (RFC 8949 Section 4.2.1-4.2.3)
- Sorts map keys in bytewise lexicographic order
- Uses the shortest encoding for each value
- Is widely supported (Rust: `ciborium` crate; TypeScript: `cbor-x`)
- Is the format used by AT Protocol, IPLD (IPFS), and COSE (IETF)

**Canonicalization rules for Benten Nodes:**

1. **Labels:** Sort alphabetically, encode as a CBOR array of strings
2. **Properties:** Sort keys lexicographically, encode as a CBOR map. Values are typed:
   - Strings: UTF-8, NFC-normalized
   - Integers: CBOR integer encoding (shortest form)
   - Floats: IEEE 754 double-precision (64-bit), canonical encoding
   - Booleans: CBOR true/false
   - Null: CBOR null
   - Arrays: CBOR arrays (recursively canonicalized)
   - Maps: CBOR maps (keys sorted, recursively canonicalized)
3. **The canonical form excludes:** anchor ID, timestamps, metadata, Edge data

### 13.3 The Hash Function

**Recommended: SHA-256 with multihash prefix**

- SHA-256 is the standard for content addressing (IPFS, Git's migration target, npm, AT Protocol)
- Multihash prefix (2 bytes: hash function identifier + digest length) enables future algorithm migration without breaking existing hashes
- Concrete encoding: `0x1220` + 32-byte SHA-256 digest = 34 bytes total
- Base encoding for human-readable display: Base32 (lowercase, no padding) -- compatible with DNS labels, URL-safe

---

## 14. The Mutable Content Problem

### 14.1 The Fundamental Tension

Content-addressed hashing works perfectly for immutable data. But most data in Benten is mutable: blog posts are edited, module settings change, user profiles are updated.

The anchor + version chain model resolves this:
- The **anchor** provides stable identity (mutable reference)
- Each **version** provides content identity (immutable snapshot)
- The **CURRENT pointer** connects mutable to immutable

This is the same pattern as:
- IPNS (mutable name -> immutable CID) in IPFS
- AT Protocol (mutable DID -> immutable commit CID)
- Holochain (mutable "original entry hash" convention -> immutable entry hash)
- Git (mutable branch name -> immutable commit hash)

### 14.2 What Gets Content-Hashed vs. What Does Not

| Element | Content-Hashed? | Rationale |
|---|---|---|
| Version Node properties | Yes | Core integrity primitive |
| Version Node itself (including parentHash, authorDID) | Yes | Chain integrity |
| Anchor Node | No | Stable identity; content is just the anchor ID |
| CURRENT pointer (Edge) | No | Mutable pointer; changes on every edit |
| NEXT_VERSION Edge | Implicitly (via parentHash in version Node) | The hash chain encodes the chain structure |
| Edges between anchors | Yes (hash of type + endpoints + properties) | Integrity of relationships |
| Binary blobs | Yes | CAS storage, deduplication |
| Operation subgraphs | Yes | Module integrity, cache keying |
| Sync metadata (vector clocks, agreement state) | No | Instance-specific, changes frequently |
| Materialized views (IVM cache) | No | Derived data, reconstructable |

### 14.3 Hash Stability Under Edits

When a user edits a blog post:
1. The post's anchor ID does not change
2. A new version Node is created with a new content hash
3. The CURRENT pointer moves to the new version
4. The old version's content hash remains unchanged
5. Any external reference to the old version's hash still resolves correctly

This means: **references by anchor ID survive edits (stable but not verifiable). References by content hash do not survive edits (unstable but verifiable).** The choice of reference type depends on the use case:

- "Link to this blog post" -> anchor reference (you want the latest version)
- "This governance vote references exactly this module version" -> content hash reference (you want immutability)
- "Verify this synced data matches what was published" -> content hash (verification)
- "Follow this user's content" -> anchor reference (you want updates)

---

## 15. Performance Considerations

### 15.1 Hash Computation Cost

SHA-256 performance on modern hardware:
- Small data (< 1KB): ~1 microsecond per hash (dominated by setup overhead)
- Medium data (1KB - 1MB): ~3 microseconds per KB (throughput-dominated)
- Large data (> 1MB): ~3 GB/s on a single core with hardware acceleration (SHA-NI instructions)

For typical Benten Nodes (properties are 100 bytes - 10KB):
- Hash computation: 1-5 microseconds per Node
- For a batch of 1,000 Nodes: 1-5 milliseconds total
- This is negligible compared to storage I/O (10-100 microseconds per write)

**Conclusion:** Hash computation is not a performance bottleneck. Even aggressive hashing (hash every Node on every write) costs less than the storage operation itself.

### 15.2 Merkle Tree Maintenance Cost

Maintaining the Merkle tree incrementally:
- One Node update: O(log N) hash computations to update the tree from leaf to root
- For a subgraph of 10,000 Nodes: ~14 hash computations per update (log2(10000) ≈ 13.3)
- At 1 microsecond per hash: ~14 microseconds per update
- At 100 writes per second: ~1.4 milliseconds total per second

**Conclusion:** Merkle tree maintenance is also not a bottleneck. It adds microseconds to each write -- well within the engine's performance targets.

### 15.3 Storage Overhead

Content hashes add 34 bytes per version Node (multihash-encoded SHA-256). For a Node with 500 bytes of properties, this is a 6.8% overhead. For a Node with 5KB of properties, it is 0.7%. For binary blobs, it is negligible.

Parent hashes (for version chain integrity) add another 34 bytes per version Node.

Signatures add ~64 bytes (Ed25519) per version Node.

Total overhead per version Node: ~132 bytes of integrity metadata. This is the cost of tamper-evidence and sync efficiency. For a system whose version Nodes are typically kilobytes, this is acceptable.

### 15.4 Sync Savings

The performance cost of hashing is repaid many times over during sync:

| Scenario | Without Hashes | With Merkle Hashing |
|---|---|---|
| 10,000 Nodes, 50 changed | Transfer: 10,000 Nodes | Transfer: 50 Nodes + ~200 hashes |
| 100,000 Nodes, 100 changed | Transfer: 100,000 Nodes | Transfer: 100 Nodes + ~340 hashes |
| 1M Nodes, 1,000 changed | Transfer: 1M Nodes | Transfer: 1,000 Nodes + ~400 hashes |

At an average of 1KB per Node, a sync of 100,000 Nodes transfers ~100MB. With Merkle hashing, the same sync transfers ~100KB (the 100 changed Nodes) + ~11KB (340 hashes). This is a 1,000x reduction.

---

## 16. Implementation Roadmap

### 16.1 Phase 1: Foundation (Build with First Engine Crates)

**Content hashing for version Nodes:**
- Add `contentHash` field to version Node structure
- Implement canonical CBOR serialization for Node properties
- Compute SHA-256 hash on version Node creation
- Store hash as version Node metadata

**Binary blob CAS:**
- Content-addressed store for files/images
- Deduplication by content hash
- Reference counting for garbage collection

### 16.2 Phase 2: Integrity (Build with Sync Protocol)

**Hash chain for version chains:**
- Add `parentHash` field to version Nodes
- Verify chain integrity on sync receive
- Violation detection and reporting

**Signed versions:**
- Add `authorDID` and `signature` fields
- Ed25519 signature of contentHash
- Signature verification on sync receive

### 16.3 Phase 3: Efficient Sync (Build with Sync Implementation)

**Merkle tree construction:**
- Build Merkle trees over sync agreement subgraphs
- Incremental maintenance via IVM
- Root hash comparison as sync fast path

**Merkle-based sync protocol:**
- Exchange roots -> drill into differences -> transfer delta
- Integration with the sync protocol specified in `critique-p2p.md`

### 16.4 Phase 4: Trust and Governance (Build with P2P Tiers)

**Module content hashing:**
- Compute hash over module definition subgraphs
- Hash comparison during sync (schema agreement verification)

**Attestation Nodes:**
- Define Attestation and Revocation Node types
- Trust graph traversal for module verification
- IVM-maintained "trusted module" materialized view

**Governance integration:**
- Hash-referenced governance proposals
- Signed vote Nodes
- Execution verification

### 16.5 Phase 5: Advanced (Post-MVP)

**Content-addressed networking:**
- Request data by hash from any peer in a Garden
- Peer selection based on network proximity
- Lazy blob loading during sync (metadata first, content on demand)

**Cross-Garden deduplication:**
- Hash-based recognition of shared content across Gardens
- Reduced storage for members of multiple Gardens

---

## 17. Relationship to Existing Architecture

### 17.1 Operation Subgraph Hashing (Already Proposed)

`operation-vocab-p2p.md` Section 11 already proposes content-addressing for operation subgraphs. This exploration extends that proposal to ALL version Nodes, binary blobs, module definitions, and trust attestations. The operation subgraph hashing is a specific case of the general principle described here.

### 17.2 Version Chains (Enhances the Existing Model)

The specification's version chain model (anchor -> CURRENT -> version Nodes -> NEXT_VERSION chain) is preserved and strengthened. Content hashes add tamper-evidence and sync efficiency without changing the fundamental model. The hash chain (parentHash linking) transforms the version chain from a "trust the pointer" model to a "verify the chain" model.

### 17.3 CRDT Sync (Enables Efficient Implementation)

The sync protocol critiqued in `critique-p2p.md` and `critique-mesh-sync.md` requires efficient delta computation. Merkle trees over subgraphs provide exactly this capability. The content hashing foundation described here is a prerequisite for the Merkle-based sync protocol.

### 17.4 Capability System (Strengthens Security)

The capability enforcement described in the specification is strengthened by content-addressed attestations. Module hash verification ensures that a module's declared capabilities match its actual definition. Governance hash references ensure that capability changes are unambiguous.

### 17.5 IVM Integration

The Incremental View Maintenance system is the natural home for Merkle tree maintenance. The Merkle tree over a subgraph is a materialized view: it is updated incrementally when the underlying data changes, and its root hash is always available in O(1). The IVM's existing infrastructure (change tracking, incremental computation, cache invalidation) directly supports Merkle tree maintenance.

---

## 18. Open Questions

### 18.1 Granularity of Hashing

Should each property within a Node have its own hash? Or is a single hash per Node sufficient?

**Per-property hashing** enables finer-grained deduplication and sync (transfer only changed properties, not the entire Node). But it adds significant overhead: a Node with 20 properties would have 20 hashes.

**Per-Node hashing** is simpler and sufficient for most use cases. The version chain already captures the full state at each version; per-property granularity is only useful for large Nodes where bandwidth is constrained.

**Recommendation:** Per-Node hashing as the default. Per-property hashing as an optional optimization for specific Node types (e.g., Nodes with large binary properties).

### 18.2 Hash Algorithm Agility

What happens when SHA-256 is eventually superseded?

With multihash encoding, new hashes use the new algorithm, and old hashes remain valid (their prefix declares which algorithm produced them). But comparison across algorithms requires rehashing: to compare a SHA-256 hash with a SHA-3 hash, you must recompute one of them.

**Recommendation:** Design the system to support mixed-algorithm hash sets, but strongly recommend a single default algorithm at any given time. Algorithm migration is a coordinated upgrade, not a runtime decision.

### 18.3 Hash Collision Handling

SHA-256 collisions are currently impossible to produce (no known attack). But defense in depth suggests planning for the possibility.

**Recommendation:** If two different Nodes produce the same hash (detected during deduplication), treat it as an error condition (not a silent merge). Log an alert. Store both Nodes with a collision marker. This is the same approach Git takes: detect collisions, do not silently corrupt data.

### 18.4 Privacy and Content Hashing

Content hashes leak information: an observer who knows the content of a Node can compute its hash and check whether a given hash matches. This enables confirmation attacks: "I suspect this encrypted subgraph contains a copy of this specific document. Let me check by comparing hashes."

For encrypted subgraphs (E2EE in Atriums), content hashes should be computed over the encrypted data, not the plaintext. This prevents hash-based confirmation attacks while still enabling integrity verification.

### 18.5 Partial Subgraph Hashing

Can you hash a subgraph that you only partially have? For example, if you have Nodes A, B, C of a subgraph but are missing Node D, can you compute a partial Merkle root?

**Yes, with caveats.** A Merkle tree can be computed with placeholder hashes for missing Nodes. The resulting root hash is useful for "the Nodes I have match what I expect" but is NOT the true root hash (which requires all Nodes). The sync protocol must distinguish between "verified partial hash" and "verified complete hash."

---

## 19. Summary: Content-Addressed Hashing as a Fundamental Primitive

Content-addressed hashing is not a feature to be added to the engine. It is a **fundamental primitive** that strengthens every other system:

| System | Without Content Hashing | With Content Hashing |
|---|---|---|
| Version chains | Trust the pointer | Verify the chain |
| Sync protocol | Transfer everything and diff | Compare Merkle roots, transfer only differences |
| Module installation | Trust the source | Verify the hash matches community attestation |
| Governance | "Upgrade to the new version" (ambiguous) | "Upgrade from hash X to hash Y" (unambiguous) |
| Data integrity | Hope nothing was modified | Recompute and verify |
| Binary storage | Store by location | Store by content (automatic deduplication) |
| Operation subgraphs | Identity by name | Identity by structure (Unison model) |
| Trust | "I trust this Garden" (vague) | "This Garden attests hash X" (verifiable) |
| Fork/rejoin | "Where did we diverge?" (expensive to compute) | "Compare Merkle roots at the fork point" (fast) |

The anchor + content hash dual identity model is the architectural foundation. Anchors provide the stable, human-friendly identity layer. Content hashes provide the cryptographic, machine-verifiable integrity layer. Together, they make every interaction in the Benten platform -- local or P2P -- tamper-evident and efficiently syncable.

**The implementation cost is minimal:** ~132 bytes of overhead per version Node, ~14 microseconds of computation per write, and a canonical serialization format (CBOR). The sync savings alone justify the investment many times over.

**The philosophical alignment is total:** In a platform where "data is owned by the user" and "either party can fork at any time," content-addressed hashing is not optional. It is the mechanism that makes ownership verifiable and forking practical.

---

## Sources

- [Content Identifiers (CIDs) | IPFS Docs](https://docs.ipfs.tech/concepts/content-addressing/)
- [CID Specification | multiformats](https://github.com/multiformats/cid)
- [Merkle DAGs | IPFS Docs](https://docs.ipfs.tech/concepts/merkle-dag/)
- [Personal Data Repositories - AT Protocol](https://atproto.com/guides/data-repos)
- [Merkle Search Tree (DavidBuchanan314 implementation)](https://github.com/DavidBuchanan314/merkle-search-tree)
- [Merkle-CRDTs: Merkle-DAGs meet CRDTs (Protocol Labs, 2020)](https://research.protocol.ai/publications/merkle-crdts-merkle-dags-meet-crdts/psaras2020.pdf)
- [DefraDB Merkle CRDT Usage](https://open.source.network/blog/how-defradb-uses-merkle-crdts-to-maintain-data-consistency-and-conflict-free-data-management-for-web3-applications)
- [Holochain DHT Architecture](https://developer.holochain.org/concepts/4_dht/)
- [Holochain Validation](https://developer.holochain.org/build/validation/)
- [Holochain Agent-Centric Approach 2025](https://defi-planet.medium.com/can-holochain-replace-traditional-blockchains-reviewing-its-agent-centric-approach-in-2025-bf48fd9f6483)
- [UCAN Specification](https://ucan.xyz/specification/)
- [Unison: The Big Idea (content-addressed code)](https://www.unison-lang.org/docs/the-big-idea/)
- [Nix Content-Addressed Derivations](https://nix.dev/manual/nix/2.30/development/experimental-features.html)
- [Merkle Trees in Git and Bitcoin](https://initialcommit.com/blog/git-bitcoin-merkle-tree)
- [Efficient Data Synchronization with Merkle Trees](https://deepengineering.substack.com/p/merkle-trees-and-anti-entropy-concepts)
- [Merkle Trees | AlgoMaster System Design](https://algomaster.io/learn/system-design/merkle-trees)
- [Cartesian Merkle Tree (2025)](https://arxiv.org/abs/2504.10944)
- [npm ssri (Standard Subresource Integrity)](https://www.npmjs.com/package/ssri)
- [Lockfile Poisoning and Hash Integrity in Node.js](https://medium.com/node-js-cybersecurity/lockfile-poisoning-and-how-hashes-verify-integrity-in-node-js-lockfiles-0f105a6a18cd)
- [Web of Trust - Wikipedia](https://en.wikipedia.org/wiki/Web_of_trust)
- [Kleppmann et al., "A Highly-Available Move Operation for Replicated Trees," IEEE TPDS, 2021](https://martin.kleppmann.com/papers/move-op.pdf)
- [Kleppmann et al., "A Conflict-Free Replicated JSON Datatype," IEEE TPDS, 2017](https://martin.kleppmann.com/papers/json-crdt-tpds17.pdf)
- [Distributed Data Deduplication for Big Data (ACM Computing Surveys, 2025)](https://dl.acm.org/doi/10.1145/3735508)
- [RFC 8949 - CBOR Deterministic Encoding](https://www.rfc-editor.org/rfc/rfc8949#section-4.2)
- [Multihash Specification](https://multiformats.io/multihash/)
