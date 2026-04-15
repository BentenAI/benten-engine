# Archive

Historical documents that informed the current design but have been superseded by committed specifications. Preserved because the ADDL methodology depends on the chain of critic findings and decisions being auditable.

## `v1-critiques/` (16 files)

The first round of critic reviews against the v1 specification (2026-04-13). Scored an average of 4.7/10. Findings drove the revision to v2.

- `critique-architecture.md` — engine architecture review
- `critique-correctness.md` — logic and edge case analysis
- `critique-data-integrity.md` — transaction/referential integrity concerns
- `critique-security.md` — auth, trust, OWASP
- `critique-developer-experience.md` — DX friction
- `critique-performance.md` — scalability and performance concerns
- `critique-composability.md` — plugin/module extensibility
- `critique-rust-practices.md` — Rust-specific best practices (drove papaya/mimalloc/thiserror choices)
- `critique-p2p.md` — P2P protocol concerns
- `critique-mesh-sync.md` — sync protocol concerns
- `critique-crdt-graph.md` — CRDT-over-graph concerns
- `critique-ecosystem.md` — ecosystem positioning
- `critique-competitive.md` — vs. Payload, Strapi, Sanity
- `critique-ai-agents.md` — AI agent integration concerns
- `critique-holochain-perspective.md` — Holochain comparison
- `critique-fresh-eyes.md` — devil's advocate review

## `v2-reviews/` (6 files)

Second round against the v2 specification (2026-04-13). Scored an average of 6.3/10. Findings drove updates to committed specs.

- `v2-review-architecture.md`
- `v2-review-completeness.md`
- `v2-review-dx.md`
- `v2-review-feasibility.md`
- `v2-review-p2p.md`
- `v2-review-security.md`

Current committed specs (ENGINE-SPEC, PLATFORM-DESIGN, BUSINESS-PLAN, CLAUDE) already incorporate the v2 findings.

## `vocab-derivation/` (8 files)

The derivation path for the 12 operation primitives (10 → 12 after multi-perspective review). Retained for the reasoning trail.

- `operation-vocab-systems.md` / `review-vocab-systems.md`
- `operation-vocab-dx.md` / `review-vocab-dx.md`
- `operation-vocab-security.md` / `review-vocab-security.md`
- `operation-vocab-p2p.md` / `review-vocab-p2p.md`

The final 12 primitives are validated empirically in `docs/validation/paper-prototype-handlers.md` (2.5% SANDBOX rate across 5 real handlers).

## `superseded/` (5 files)

Earlier specifications and exploration docs whose conclusions are now committed to the canonical specs.

- `v1-specification.md` — original unified spec (superseded by ENGINE-SPEC + PLATFORM-DESIGN + BUSINESS-PLAN split)
- `v2-unified-specification.md` — unified v2 spec (same supersession)
- `explore-self-evaluating-graph.md` — code-as-graph paradigm analysis (conclusion committed to ENGINE-SPEC)
- `explore-content-addressed-hashing.md` — hashing scheme analysis (conclusion: BLAKE3 + DAG-CBOR + CIDv1, committed to ENGINE-SPEC)
- `explore-blockchain-assessment.md` — "should we use a blockchain?" analysis (conclusion: no, committed to VISION and BUSINESS-PLAN)

## Why Keep These

1. **ADDL methodology** — the process requires auditable chains of findings and decisions. New agents joining the project can trace *why* each decision was made.
2. **Critic review precedent** — when running new rounds of critics, reviewers can read the history of what prior critics found and how it was addressed.
3. **Honest history** — the project has gone through significant revisions; hiding that would obscure how the current design emerged.

## When to Delete

Never. If content is truly obsolete, note it in the relevant file header but keep the file. History is cheap; regret at lost reasoning is not.
