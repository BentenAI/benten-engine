# Security Posture — Benten Engine Phase 1

This document records the security claims Benten makes in Phase 1 and the known compromises those claims rest on. Each compromise is tied back to `.addl/phase-1/00-implementation-plan.md` (R1 Triage Addendum); this document is the written, referenceable form.

## Named Compromises

### Compromise #6 — BLAKE3 128-bit effective collision resistance

Benten uses **BLAKE3-256** with a 32-byte digest embedded in every CIDv1. The academic collision-resistance bound for any cryptographic hash is `2^(n/2)` (birthday bound), giving BLAKE3-256 a **128-bit effective collision resistance**. This is the bound that every Benten Phase-1 security argument rests on — NOT the full `2^256` preimage bound.

**Where this matters:**

- **Content-addressed Nodes (`Cid`).** A collision would allow a malicious writer to forge a Node that hashes to the same CID as a legitimate Node — a "masquerade" attack. 128-bit resistance requires ~`2^128` hashes to find a collision; infeasible under any classical threat model.
- **Version-chain `prior_head` threading** (`benten_core::version::append_version`). The API uses CIDs to name the head each writer observed. A collision on a CID used as `prior_head` could, in principle, let an attacker smuggle an alternative chain past the fork-detection check. The same 128-bit bound applies.
- **Phase 3 UCAN-by-CID.** Phase 3 will reference capability grants by CID. Revoke-by-CID paths assume the CID of a grant is unique; again, 128-bit collision resistance is the assumption.

**What this posture does NOT claim:**

- **Quantum resistance.** Grover's algorithm reduces the effective collision bound to `2^64` under a quantum adversary. This is still infeasible for the current state of quantum hardware, but it is no longer "categorically" secure. A post-quantum hash option is a Phase N+ consideration; BLAKE3 is not post-quantum.
- **Second-preimage resistance stronger than 128 bits.** For adversaries who already know a target CID and wish to construct a colliding Node, the dominant-term bound is still ~`2^128`. Benten does not rely on the higher 256-bit preimage bound.

**Phase 2 action items:**

- Mirror this posture into end-user docs (`docs/QUICKSTART.md` security section).
- Document the same assumption in the TypeScript wrapper's JSDoc for `@benten/engine` node-creation APIs.
- When Phase 3 introduces the UCAN-by-CID path, restate the bound at that integration point.

---

### Compromise #2 — `E_CAP_DENIED_READ` leaks existence (Option A)

Phase 1 ships the stricter error route: a denied read returns `CapError::DeniedRead { required, entity }` (code `E_CAP_DENIED_READ`) rather than silently returning `None` (which would be indistinguishable from not-found).

**Why:** Option A surfaces the denial to the caller, enabling the application layer to distinguish "you don't have access" from "this resource doesn't exist." This leaks the fact that the Cid EXISTS in the backend to an unauthorized reader.

**Attack class:** an unauthorized reader can probe the CID space to enumerate what is stored, without ever reading contents.

**Phase 3 revisit:** sync + federation will revisit this. Options: (a) return indistinguishable `None` (the option-B plan); (b) add a capability level that permits existence-check but not read; (c) rotate CIDs per-reader (privacy-preserving addressing).

**Regression test:** `option_a_existence_leak_is_documented_compromise` greps this doc for `option A` + `E_CAP_DENIED_READ` — keeping this section load-bearing for the test.

---

*Future compromises with security implications will be appended as sections here, each tagged with the compromise number from the R1 Triage Addendum.*
