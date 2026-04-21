# Future Scope

**Status:** Architectural proposals that are NOT committed. Each item here is a candidate product or feature evaluated after the committed Phase 1-8 scope ships to real users.

This directory exists because several critic reviews (architecture-purity, engine-philosophy, fresh-eyes, competitive-analysis, DX) independently observed that exploratory scope in the committed specifications was pressuring the engine toward premature generality. Moving these ideas here preserves the thinking without letting them act as forward-guidance on Phase 1-8 abstractions.

## What's In Here

- **`phase-2-backlog.md`** — Consolidated list of items deferred from Phase 1 that have a clear Phase 2 landing point (arch-1 dep break, 4 deferred primitives, 6 deferred invariants, IVM generalisation, security follow-ups). Distinct from the exploratory proposals below: backlog = committed deferrals with concrete Phase 1 references; `future/` proper = exploratory, may or may not ship.
- **`benten-runtime.md`** — Benten Runtime as WinterTC-compliant infrastructure product (competes with Cloudflare Workers)
- **`bentend-daemon.md`** — General-purpose compute daemon with Nomad-style pluggable drivers (containers, VMs, WASM workloads beyond ours)
- **`compute-marketplace.md`** — Broad peer-to-peer compute marketplace for arbitrary workloads (hardware-renting for containers/VMs beyond Benten's own compute)

Additional unwritten-but-exploratory items (covered in [`FULL-ROADMAP.md`](../FULL-ROADMAP.md)):
- Full Groves (fractal/polycentric governance beyond Gardens MVP)
- Garden/Grove federation (cross-community sync, parent authority domains)
- Knowledge attestation marketplace (speculative attestation, AI trust signals)
- DAO transition (four-phase shift from sole operator to community-governed foundation)
- Governance Grove (meta-community that governs the platform itself)

## What Moved OUT of `future/` on 2026-04-14

After Ben's articulation of the three-pillar vision:

- **Benten Credits (basic)** → promoted to Phase 8 committed. Credits are the primary revenue mechanism; they need to be committed.
- **Personal AI Assistant** → promoted to Phase 6 committed. It's the adoption driver.
- **Digital Gardens (MVP)** → promoted to Phase 7 committed. Basic community spaces are core to the vision.
- **Platform features** (schema-driven rendering, self-composing admin, plugin manifests) → promoted to Phase 5 committed.
- **Three-products framing** → deleted entirely. Superseded by the three-pillar framing in [`VISION.md`](../VISION.md).

## The Principle

Move a proposal from `future/` to `research/` when we're actively deciding on it. Move from `research/` to a committed spec when the design is locked and it enters a phase plan. Never the other way — once a spec is committed, downgrading it means we over-committed and need to be honest about that.

## What Committed Really Means

A design is committed when:
1. It's in a canonical spec (VISION, ARCHITECTURE, ENGINE-SPEC, PLATFORM-DESIGN, BUSINESS-PLAN, DSL-SPECIFICATION) as a stated capability
2. It has a defined phase in [`FULL-ROADMAP.md`](../FULL-ROADMAP.md)
3. Phase 1 abstractions depend on it or are designed to support it

Under this definition, as of 2026-04-14, committed = Phases 1-8 (engine, sync, Thrum, platform features, AI Assistant, Gardens MVP, Credits). Everything else here.

## Revival Criteria

For any proposal here to move into committed scope, it must meet all four:
1. Phase 1-8 has shipped and external users depend on it
2. Concrete demand exists for this specific feature
3. A dedicated owner (team, founder, or funded contributor) can commit to the scope
4. The critic review that kept it exploratory has been revisited with new information
