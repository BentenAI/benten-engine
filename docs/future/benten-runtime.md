# Benten Runtime (Future / Exploratory)

**Status:** Candidate product. Not committed. Architectural proposal extracted from 2026-04-14 vision evolution.

## The Idea

A WinterTC-compliant edge runtime that runs the Benten engine (compiled to WASM) as a host for communities. Rather than deploying Benten communities *inside* proprietary edge runtimes (Cloudflare Workers), peers run their own Benten Runtime instances. This creates a peer-distributed alternative to proprietary edge platforms.

## Why It's Interesting

- WinterCG became Ecma TC55 in December 2024 — real cross-platform standard
- napi-rs v3 compiles to wasm32-wasip1-threads from the same Rust codebase
- Open-source foundations available (Deno, workerd, wasmtime)
- The broader three-pillar business model ([`../VISION.md`](../VISION.md)) makes this a potential infrastructure product if the engine + adoption pillars drive demand for self-hosted alternatives to Cloudflare Workers

## Why It's Deferred

- No customer demand yet — communities can deploy on Cloudflare Workers today
- Runtime quality is existential for adoption; half-built is worse than not built
- Two build targets (native + WASM) from day one is fine; building a full hosting product on top is a separate company
- The engine needs to ship first

## Minimum Viable Runtime (if revived)

- WinterTC Minimum Common API surface
- Content-addressed fetch integration (pulls from peer network)
- Operation subgraph evaluator (Benten engine WASM build)
- Capability enforcement at the runtime level
- Stateful coordinator primitive (SQLite-backed, Durable-Object-like)
- Metering/billing integration with compute marketplace
- Peer discovery

## Foundation Options

| Option | Tradeoff |
|--------|----------|
| Fork Deno | Mature, MIT, has Deno KV — couples us to Deno's roadmap |
| Fork workerd | Cloudflare's open-source Workers runtime — closest to Cloudflare-compat |
| Build on wasmtime + custom WinterTC shim | Maximum control, maximum work |

## Known Hard Problems

- Threading: WASI Preview 2 has no threading; `wasi-threads` proposal withdrawn
- Storage: redb requires filesystem I/O, doesn't work inside WASM
- Durable Object equivalent: only Cloudflare has this natively in 2026
- P2P sync: iroh needs raw sockets, won't run inside a Worker / DO

## When to Revive

Conditions for moving this proposal from `future/` to `research/`:
1. Phase 1 engine has 100+ external users
2. At least 10 of those users request self-hosted alternative to Cloudflare
3. A founding team or contributor is willing to own the runtime product
4. NextGraph or similar competitor has NOT commoditized the category first

## Source Material

See `docs/research/explore-distributed-compute-vision.md` section 8.5 for the full thinking that produced this proposal.
