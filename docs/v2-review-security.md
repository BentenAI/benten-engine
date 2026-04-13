# Security Review: Benten Platform Specification v2

**Reviewer:** Security & Trust Auditor
**Date:** 2026-04-11
**Scope:** `docs/BENTEN-PLATFORM-SPECIFICATION.md` (v2) evaluated against the 5 critical vulnerabilities from `docs/critique-security.md` (v1), OWASP Top 10:2025, and new attack surfaces introduced by token economics, governance, identity verification, and decentralized KYC.

---

## Security Score: 7 / 10

**Rationale:** v2 represents a significant architectural improvement over v1. Three of the five original criticals are addressed through structural redesign (Cypher injection eliminated by operation subgraphs, system-zone boundary codified as invariant #11, IVM bounded by DAG constraints). One critical is partially addressed (CRDT revocation-wins policy for capability edges). One critical remains unaddressed (HLC clock manipulation). However, v2 introduces substantial new attack surface through token economics, governance mechanisms, and decentralized identity -- each with its own class of vulnerabilities that the specification does not address.

| v1 Score | v2 Score | Delta |
|----------|----------|-------|
| 5 / 10 | 7 / 10 | +2 |

Breakdown:
- Capability model design: 8/10 (up from 7 -- UCAN-native, operator-configured, same system for all actors)
- Sync trust model: 4/10 (up from 3 -- revocation-wins for capability edges, but clock manipulation unaddressed)
- Injection surface management: 8/10 (up from 2 -- operation subgraphs eliminate Cypher injection as a primary surface)
- Memory safety / resource exhaustion: 7/10 (up from 4 -- 14 structural invariants with configurable bounds)
- Sandboxing: 7/10 (up from 6 -- SANDBOX primitive is well-specified: no re-entrancy, fuel-metered, time-limited, max output)
- Cryptographic foundations: 7/10 (up from 4 -- Ed25519, BLAKE3, CBOR, UCAN, did:key all committed)
- Token economics security: 3/10 (new -- significant underspecification)
- Governance attack resistance: 3/10 (new -- significant underspecification)
- Identity/KYC security: 4/10 (new -- marketplace model introduces trust bootstrapping problems)

---

## v1 Critical Vulnerability Disposition

### CRITICAL-1 (v1): Cypher Query Injection via `engine.query()` -- RESOLVED

v2 eliminates the `engine.query(cypher: string)` API entirely. All computation is expressed as operation subgraphs composed from 12 typed primitives. There is no raw Cypher surface in the core API. Section 9 (Open Questions, item 2) asks "Do we need a Cypher parser?" but frames it as optional and additive, not as the primary API.

**Residual risk:** The GATE primitive (Section 2.2, #7) is described as a "custom logic escape hatch." If GATE allows arbitrary computation (not just capability checks), it could reintroduce injection-equivalent vulnerabilities depending on what "custom logic" means at the engine level. The specification describes GATE as "for complex validation/transformation that can't be expressed as TRANSFORM" with "capability checking via `requires` property on any Node" -- but does not define what GATE can actually execute. This is tracked as NEW-5 below.

**Verdict:** The primary injection vector is gone. GATE is a residual concern.

### CRITICAL-2 (v1): Capability Graph Queryable/Mutable via Same Primitives -- RESOLVED

v2 codifies the system-zone/user-zone boundary as Structural Invariant #11: "System-zone labels unreachable from user operations -- Kernel/userspace boundary." This directly addresses the v1 recommendation for an explicit two-zone model.

Additionally, Invariant #13 ("Immutable once registered") and Invariant #12 ("Registration-time structural validation") close the TOCTOU vector where a subgraph could be modified between validation and execution.

**Residual risk:** The specification states the invariant but does not describe the enforcement mechanism. Is it enforced at the query planner level? At the evaluator level? Both? The v1 recommendation was explicit: "The Cypher parser/planner must enforce zone boundaries at the query planning stage, not at runtime." Since v2 uses an evaluator rather than a query parser, the equivalent would be: the evaluator must reject any operation Node whose target resolves to a system-zone label. The specification should state this explicitly.

**Verdict:** Structurally resolved. Enforcement mechanism needs specification.

### CRITICAL-3 (v1): CRDT Clock Manipulation -- PARTIALLY ADDRESSED

v2 adds one important improvement: "Edges: add-wins with per-edge-type policies (capability revocation MUST win)" (Section 3.2). This directly addresses the edge spam attack vector for capability revocation edges -- revocation will not be trumped by concurrent adds.

However, the core attack vector -- HLC clock manipulation for LWW property conflicts -- is **completely unaddressed**. v2 still uses "per-field last-write-wins with Hybrid Logical Clocks" (Section 3.2) with no mention of:
- Clock skew bounds (the v1 P0 recommendation)
- Per-peer rate limits
- Peer reputation scoring
- Tombstone-wins mode for edges where integrity matters more than availability

The v1 specification said "non-deterministic values captured in version chain, not replayed." v2 repeats this same language verbatim with "(non-deterministic values captured in version chain, not replayed)" -- but does not add any clock validation.

**Verdict:** Capability revocation edge policy is a meaningful improvement. The HLC clock manipulation attack remains fully exploitable. A malicious peer can still set its clock to year 2099 and overwrite any property on any synced node.

### CRITICAL-4 (v1): UCAN Revocation Propagation in P2P -- NOT ADDRESSED

v2 does not describe a revocation propagation protocol. The "capability revocation MUST win" edge policy (Section 3.2) helps with the local merge semantics -- when a revocation edge and a grant edge conflict, the revocation wins. But this does not solve the propagation problem:

1. How does Instance C learn about a revocation that happened on Instance A, when C only communicates through Instance B (whose capability was revoked)?
2. What happens when a peer is offline during revocation and comes back with stale grants?
3. Are revocation records prioritized in the sync protocol?
4. What is the maximum window during which a revoked capability can still be exercised?

None of the v1 recommendations (short-lived grants with renewal, revocation as first-class sync primitive, offline revocation buffer) appear in v2.

**Verdict:** Unaddressed. The local merge semantics improvement (revocation-wins) does not solve the distributed revocation propagation problem.

### CRITICAL-5 (v1): IVM Resource Exhaustion -- SUBSTANTIALLY ADDRESSED

v2 addresses this through multiple structural invariants:
- Invariant #1: Subgraphs are DAGs (no cycles) -- prevents recursive view definitions
- Invariant #2: Max depth configurable per capability grant -- bounds traversal depth
- Invariant #3: Max fan-out per node -- prevents combinatorial explosion
- Invariant #5/6: Max nodes (4096) and edges (8192) per subgraph -- bounds view definition size
- Invariant #8: Cumulative iteration budget (multiplicative) -- prevents nested loop explosion
- Invariant #12: Registration-time structural validation -- rejects malformed views before they execute

Since IVM views are defined as operation subgraphs (Section 2.7), they inherit all these bounds. A malicious module cannot define a view with unbounded traversal depth because the subgraph itself is bounded.

**Residual risk:** The v1 recommendation for per-view CPU/memory budgets and circuit breakers is not explicitly addressed. The structural invariants bound the definition complexity but do not bound the data volume a view might touch. A view defined over a legitimate-looking pattern could still be expensive if the data set is large. The SANDBOX primitive has fuel-metering and time limits, but IVM updates are not described as sandboxed.

**Verdict:** Substantially addressed through structural invariants. Runtime resource budgets for IVM update execution remain unspecified.

---

## New Vulnerabilities Introduced by v2

### NEW-1 (CRITICAL): Token Economics -- Mint/Burn Oracle Attack

**Location:** Section 5.1, 6.1

The specification states: "User sends $1 USD -> BentenAI mints 1 credit" and "User redeems credit -> BentenAI burns it, returns $1 USD." Revenue comes from investing reserves in Treasury bonds.

This creates a classic stablecoin "bank run" vulnerability:

1. **Reserve verification gap.** The specification says "1:1 reserves" and mentions "monthly attestation" under GENIUS Act requirements (Section 6.4). Monthly attestation means there is a 30-day window during which reserves could be undercollateralized without detection. If BentenAI invests reserves in instruments that lose value (even Treasury bonds carry mark-to-market risk), the 1:1 peg breaks.

2. **Mint authority is a single point of compromise.** BentenAI is the sole minter/burner in Phase 1-2. A compromised mint key can create unlimited credits. The specification does not describe:
   - Multi-signature requirements for mint operations
   - Rate limits on minting (max credits minted per time period)
   - Separation between the investment function (Treasury bonds) and the operational function (mint/burn)
   - What happens if FedNow rails are suspended (bank closure, regulatory action)

3. **FedNow settlement risk.** FedNow transactions are instant and irrevocable. A fraudster who obtains credits through social engineering or account takeover can burn them for instant USD. Unlike credit card transactions, there is no chargeback mechanism. The specification does not describe fraud detection or cooling periods on large burn operations.

**Severity:** Critical for any deployment with real money. The economic primitives are described at the level of a whitepaper, not a security specification.

### NEW-2 (CRITICAL): Governance Attack -- Hostile Takeover via Fork Semantics

**Location:** Sections 4.3, 4.5

The specification states: "Any participant can fork any subgraph at any time, keeping full history." And governance is "configurable per community" with voting mechanisms including "token-weighted" and "liquid delegation."

Attack vectors:

1. **Governance capture via liquid delegation accumulation.** In liquid delegation, voters can delegate their votes to representatives. An attacker gradually accumulates delegations from inactive members (who delegated to "whoever the community admin recommends") and then uses the accumulated voting power to change governance rules -- including changing the meta-governance rules that govern how governance is changed. Once meta-governance is captured, the attacker can lock out legitimate governance participants.

2. **Fork-bomb as denial of governance.** "Fork is a right" means an attacker can repeatedly fork a community, creating confusion about which fork is "real." If the forked community has external integrations (API consumers, payment destinations), the fork creates ambiguity about where payments should go and which API endpoint is authoritative. The specification describes fork-and-compete as evolutionary pressure, but does not describe protections against fork-and-confuse.

3. **Polycentric authority conflict escalation.** Section 4.4 describes "MULTIPLE parent Groves simultaneously" with conflict resolution via "explicit priority, union (strictest wins), local override, or mediation." An attacker who controls one parent Grove can use the polycentric structure to inject restrictive rules into a child Grove through the "union (strictest wins)" resolution mode -- effectively vetoing the child Grove's governance without being a member of it.

**Severity:** Critical for any community with real governance stakes (treasury, moderation authority, membership control).

### NEW-3 (HIGH): Identity Verification -- Sybil Attacks on Attestation Marketplace

**Location:** Sections 5.2, 5.5

The knowledge attestation marketplace creates economic incentives for Sybil attacks:

1. **Attestation fee farming.** If attesting to knowledge costs a fee and "fees flow to existing attestors," an attacker who is among the first attestors of a piece of knowledge receives fees from all subsequent attestors. By creating multiple Sybil identities and being the first to attest to many knowledge nodes, the attacker accumulates a position as a fee recipient across the knowledge graph. Each subsequent legitimate attestor pays fees to the attacker's Sybil identities.

2. **KYC marketplace trust bootstrapping.** "Communities decide which verifiers they trust" creates a bootstrapping problem: who verifies the verifiers? If a malicious actor registers as a KYC provider and a community trusts that provider, all identities "verified" by that provider are treated as legitimate. The specification says "BentenAI maintains approved verifier list for token system compliance" -- but community-chosen verifiers may not be on this list, creating a two-tier system where token-system KYC is reliable but community-KYC is gameable.

3. **Credential replay across communities.** Verifiable Credentials stored as Nodes in the user's graph can be presented to any community. If Community A issues a "trusted member" credential, that credential can be presented to Community B as evidence of trustworthiness -- even if the communities have completely different trust standards. The specification does not describe credential audience restrictions (W3C VCs support this, but the specification does not mention it).

**Severity:** High. Sybil resistance is the foundational security property for any system with economic incentives, and the specification's identity model relies on a marketplace of verifiers without specifying how the marketplace itself is secured.

### NEW-4 (HIGH): GATE Primitive as Turing-Complete Escape Hatch

**Location:** Section 2.2, primitive #7

The GATE primitive is described as: "Custom logic escape hatch. For complex validation/transformation that can't be expressed as TRANSFORM. Capability checking via `requires` property on any Node."

The specification's security model depends on subgraphs being non-Turing-complete DAGs with bounded iteration (Section 2.2: "The vocabulary is deliberately NOT Turing complete -- subgraphs are DAGs with bounded iteration. This guarantees termination, enables static analysis, and prevents denial-of-service."). But GATE is explicitly defined as an escape hatch for logic that "can't be expressed" within the safe primitives.

Questions the specification does not answer:
1. What can GATE execute? Is it Rust-native code? JavaScript? A restricted expression language?
2. Does GATE have the same fuel-metering and time limits as SANDBOX?
3. Can GATE perform I/O (network, filesystem)?
4. If GATE can execute arbitrary Rust-native code, does it bypass the structural invariant guarantees?
5. Is GATE available to community-tier modules, or only platform modules?

If GATE allows arbitrary computation without resource bounds, it reintroduces the denial-of-service vector that the 12-primitive restriction was designed to prevent. If GATE is as restricted as TRANSFORM, it's unclear why it exists as a separate primitive.

**Severity:** High. The security guarantees of the entire operation-subgraph model depend on GATE being bounded, but the specification does not define what GATE can do.

### NEW-5 (HIGH): Execution Policy "leader-elected" Creates a Trusted Third Party

**Location:** Section 3.2

"Execution assignment: Handlers have an `executionPolicy`: origin-only, local, leader-elected (one designated instance runs, others receive results)."

The "leader-elected" policy means one instance executes and all others trust its results. This creates several attack vectors:

1. **Compromised leader fabricates results.** If the elected leader is compromised, it can return fabricated results for any handler. All other instances accept these results as authoritative. The specification does not describe result verification, attestation by the leader, or majority-vote confirmation.

2. **Leader election manipulation.** The specification does not describe the leader election mechanism. If it's based on reputation, capability, or voting, it inherits the governance attack vectors from NEW-2. If it's round-robin, a malicious peer can predict when it will be leader and prepare attacks.

3. **Denial of service via leader failure.** If the elected leader goes offline, all instances waiting for results are blocked. The specification does not describe leader timeout, re-election, or failover.

**Severity:** High. This is the classic Byzantine Generals problem, and the specification does not describe a BFT protocol.

### NEW-6 (HIGH): Content-Addressed Hashing Excludes Timestamps and Edges

**Location:** Section 2.6

"What gets hashed: labels + properties (NOT anchor ID, NOT timestamps, NOT edges)."

Excluding edges from the hash means two version Nodes with identical properties but completely different edge structures (different GRANTED_TO edges, different ATTESTED_BY edges, different parent-child relationships) will have the same hash. This undermines deduplication and integrity verification:

1. **Edge tampering is undetectable via hash comparison.** A malicious peer can modify edge structure (adding or removing relationships) without changing the content hash. "Same hash = same content" is only true for node properties, not for the full graph structure.

2. **Deduplication across instances conflates structurally different graphs.** If Instance A has Node X with edges {A, B, C} and Instance B has Node X with edges {A, D}, their hashes are identical. Sync deduplication will treat them as the same, potentially losing edge {B, C} or {D} depending on merge order.

**Severity:** High. The specification claims hashes are used for "Sync integrity verification" and "Deduplication across instances" but the hash does not cover the full graph state.

### NEW-7 (MEDIUM): No Supply Chain Security for Rust Crates

**Location:** Section 2.10

The crate structure lists dependencies (redb, etc.) but v2 -- like v1 -- does not mention dependency auditing, `cargo-vet`, `cargo-deny`, or any supply chain verification strategy. This was flagged in the v1 review (OWASP A03) and remains unaddressed.

### NEW-8 (MEDIUM): Compute Marketplace Verification Gap

**Location:** Section 5.3

"Verification of computation through verifiable services (storage, bandwidth) initially, general compute later."

"General compute later" means the Phase 1 compute marketplace cannot verify arbitrary computation. A malicious compute provider can accept payment, claim to have executed the computation, and return fabricated results. The specification acknowledges this gap ("verifiable services initially") but does not describe what protections exist in the interim.

---

## OWASP Top 10:2025 Checklist

| # | Category | Status | Notes |
|---|----------|--------|-------|
| A01 | Broken Access Control | IMPROVED | Capability model is now UCAN-native with operator configuration. System-zone boundary is a declared invariant (#11). Capability revocation wins over adds for edge conflicts. Still no specification for user-level access control flows (deferred to TypeScript layer). |
| A02 | Security Misconfiguration | IMPROVED | 14 structural invariants with configurable bounds provide defense-in-depth. Defaults specified (4096 nodes, 8192 edges, 1MB SANDBOX output). Open Questions reduced from v1 (cryptographic primitives committed). |
| A03 | Software Supply Chain Failures | NOT ADDRESSED | No mention of dependency auditing, cargo-vet, cargo-deny, or supply chain verification. Same gap as v1. |
| A04 | Cryptographic Failures | SUBSTANTIALLY IMPROVED | Ed25519, BLAKE3, CBOR (RFC 8949), Multihash, UCAN, did:key all committed (Section 2.11). KERI AID or did:plc for persistent identity. Key rotation mentioned via "persistent identity survives key rotation" (Section 3.3) but key rotation protocol not specified. |
| A05 | Injection | RESOLVED | Operation subgraphs eliminate Cypher injection surface. GATE and SANDBOX are the only escape hatches, both capability-gated. Cypher is an optional additive feature (Open Questions #2), not the primary API. |
| A06 | Insecure Design | PARTIAL | The 12-primitive vocabulary with DAG constraints is a strong secure-by-design decision. But GATE as an undefined escape hatch (NEW-4), leader-elected execution without BFT (NEW-5), and the edge-excluding hash (NEW-6) are design-level weaknesses. |
| A07 | Authentication Failures | PARTIAL | Three-layer identity stack is well-designed (persistent + transport + address). W3C Verifiable Credentials for attestation. But the verifier marketplace trust bootstrapping problem (NEW-3) means authentication quality depends on community-chosen verifiers of unknown reliability. |
| A08 | Vulnerable and Outdated Components | NOT ADDRESSED | Same gap as v1. No dependency management strategy described. |
| A09 | Security Logging & Alerting | IMPROVED | Invariant #14 ("Causal attribution on every evaluation -- Unsuppressible audit trail") addresses audit logging at the engine level. Audit is a composed pattern (EMIT to audit channel). But no specification of alerting, anomaly detection, or security event correlation. |
| A10 | Mishandling of Exceptional Conditions | PARTIAL | Typed error edges on operations (ON_NOT_FOUND, ON_EMPTY, ON_CONFLICT, ON_DENIED) provide structured error handling. Compensate pattern for transaction failure. But no specification for: what happens when IVM update fails, what happens when CRDT merge encounters corruption, how the evaluator handles malformed system-zone nodes. |

---

## Plugin / Module Trust Analysis (v2)

v2 replaces the fixed 4-tier trust model (platform/verified/community/untrusted) with operator-configured capability grants. This is architecturally superior -- capabilities are more granular than tiers. However:

| Attack | Without Capability | With Scoped Capability | Notes |
|--------|--------------------|----------------------|-------|
| Read other modules' data | Blocked | Blocked (scope mismatch) | Improved: capabilities scope to specific resources |
| Modify other modules' data | Blocked | Blocked (scope mismatch) | Improved: WRITE checks capabilities |
| Escalate capabilities | Blocked (invariant #11) | Blocked (attenuation = narrow only) | Resolved: system-zone boundary |
| DoS via IVM | Bounded (invariants #2-8) | Bounded | Substantially improved |
| Exfiltrate via SANDBOX | Blocked (no re-entrancy, no I/O) | Blocked | Improved: SANDBOX is well-specified |
| GATE abuse | Unknown | Unknown | NEW-4: GATE semantics undefined |
| Forge sync operations | Blocked (no sync capability) | Possible if sync capability granted | Same as v1 |

**Key improvement:** The shift from tiers to capabilities means "verified module" no longer automatically gets raw:sql/raw:cypher. Capability grants are operator-configured, so the v1 concern about "verified modules have too much power" is addressed by design -- operators choose exactly what each module can do.

**Remaining concern:** The specification does not describe capability grant templates or presets for common module types. Without good defaults, operators will either over-grant (security risk) or under-grant (usability failure).

---

## Summary of Findings

### v1 Criticals Disposition

| v1 Finding | v2 Status | Score |
|------------|-----------|-------|
| CRITICAL-1: Cypher injection | RESOLVED (operation subgraphs) | Fixed |
| CRITICAL-2: System-zone boundary | RESOLVED (invariant #11) | Fixed |
| CRITICAL-3: CRDT clock manipulation | PARTIALLY ADDRESSED (revocation-wins, but HLC still exploitable) | Half-fixed |
| CRITICAL-4: UCAN revocation propagation | NOT ADDRESSED | Open |
| CRITICAL-5: IVM resource exhaustion | SUBSTANTIALLY ADDRESSED (14 invariants) | Fixed |

### New Findings (8 total)

| # | Severity | Finding | Section |
|---|----------|---------|---------|
| NEW-1 | CRITICAL | Mint/burn oracle attack -- single-point mint authority, no multi-sig, no rate limits, no fraud detection on burn | 5.1, 6.1 |
| NEW-2 | CRITICAL | Governance hostile takeover -- liquid delegation capture, fork-bomb confusion, polycentric authority injection | 4.3, 4.5 |
| NEW-3 | HIGH | Sybil attacks on attestation marketplace -- fee farming, unverified verifiers, credential replay | 5.2, 5.5 |
| NEW-4 | HIGH | GATE primitive undefined -- potential Turing-complete escape hatch bypassing DAG safety guarantees | 2.2 |
| NEW-5 | HIGH | Leader-elected execution without BFT -- fabricated results, election manipulation, no failover | 3.2 |
| NEW-6 | HIGH | Content hash excludes edges -- edge tampering undetectable, deduplication conflates different graphs | 2.6 |
| NEW-7 | MEDIUM | No supply chain security for Rust crates (unchanged from v1) | 2.10 |
| NEW-8 | MEDIUM | Compute marketplace cannot verify general computation in Phase 1 | 5.3 |

---

## Prioritized Recommendations

### P0: Must resolve before implementation begins

1. **Define GATE primitive semantics and resource bounds** (NEW-4). State explicitly: what can GATE execute, is it fuel-metered like SANDBOX, can it perform I/O, and is it available to all capability levels. Without this, the security guarantees of the 12-primitive model are hollow.

2. **Add HLC clock skew bounds to the sync specification** (CRITICAL-3 carryover). This is a one-line addition: "Receiving peers MUST reject HLC timestamps more than [configurable, default 5 minutes] ahead of local clock." This was a P0 in the v1 review and remains unaddressed.

3. **Define the system-zone enforcement mechanism** (CRITICAL-2 residual). Invariant #11 states the policy; specify whether enforcement happens at the evaluator level, at the storage level, or both.

### P1: Must resolve before P2P sync ships

4. **Design revocation propagation protocol** (CRITICAL-4 carryover). The v1 recommendations (short-lived grants with renewal, revocation as priority sync primitive, offline revocation buffer) remain valid and unaddressed.

5. **Specify result verification for leader-elected execution** (NEW-5). At minimum: leader must sign results with its Ed25519 key, and instances must be able to demand re-execution if results are suspect.

6. **Decide whether content hashes should include edge structure** (NEW-6). If edges are excluded for performance, document the security implications and specify a separate mechanism for edge integrity verification during sync.

### P2: Must resolve before token economics ships

7. **Specify mint/burn security controls** (NEW-1). Multi-signature requirement for mint operations, per-period rate limits, cooling period on large burn operations, real-time reserve attestation (not monthly), and contingency for FedNow rail suspension.

8. **Specify Sybil resistance for attestation marketplace** (NEW-3). Options: proof-of-stake (attestors risk losing their stake if attestation is disputed), reputation scoring (weight of attestation proportional to attestor history), or minimum KYC level for attestors.

9. **Specify governance anti-capture mechanisms** (NEW-2). Options: delegation decay (delegations expire if not renewed), fork-naming conventions (the "canonical" fork is determined by membership count, not by who forks first), and parent-Grove authority limits (maximum scope of rules a parent can impose on a child).

### P3: Must resolve before production deployment

10. **Add supply chain security** (NEW-7). Integrate cargo-deny for license and vulnerability checking, cargo-vet for dependency review.

11. **Add per-peer rate limits and reputation to sync protocol** (CRITICAL-3 carryover). Beyond clock bounds, limit the volume of operations any single peer can generate per time window.

12. **Specify credential audience restrictions** (NEW-3). Verifiable Credentials should include audience claims to prevent replay across communities with different trust standards.

---

## Sources

- [OWASP Top 10:2025](https://owasp.org/Top10/2025/en/)
- [OWASP Top 10:2025 Introduction](https://owasp.org/Top10/2025/0x00_2025-Introduction/)
- [OWASP Top 10 2025 Key Changes (Aikido)](https://www.aikido.dev/blog/owasp-top-10-2025-changes-for-developers)
- [OWASP Top 10 2025 What's Changed (GitLab)](https://about.gitlab.com/blog/2025-owasp-top-10-whats-changed-and-why-it-matters/)
