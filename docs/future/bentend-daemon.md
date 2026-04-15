# `bentend` Peer Daemon (Future / Exploratory)

**Status:** Candidate product. Not committed. Architectural proposal extracted from 2026-04-14 vision evolution.

## The Idea

A single Rust daemon installable on any Linux that orchestrates heterogeneous compute workloads (containers, VMs, WASM, native) for a peer-to-peer compute marketplace. Peers install `bentend`, advertise hardware resources (CPU, GPU, RAM, storage, bandwidth), and earn Benten Credits by accepting jobs. The graph IS the control plane — jobs are Nodes, capabilities gate execution, the marketplace is bid/ask subgraphs.

## Why It's Interesting

- Turns every peer with spare hardware into a participant in the compute economy
- Unifies storage + compute + bandwidth into one marketplace (rather than three separate protocols)
- Competes with Akash/Flux/Render but without blockchain consensus overhead
- The graph-as-control-plane model is genuinely novel

## Why It's Deferred

This is Phase 5+ minimum under even the most aggressive roadmap. Specifically:
- Requires the engine, sync protocol, and credits to all be working first
- Compute verification (Proof of Sampling with reputation) is unsolved at scale
- Container orchestration **does not cleanly compose from the 12 operation primitives** — it's side-effect-heavy, needs log streaming, signal propagation, zombie reaping. Either SANDBOX becomes very rich (collapsing toward "we built a runtime") or we need a 13th primitive
- Trust tier primitives need to be production-tested for capability isolation between tenants
- The economic layer (Credits, tab-based settlement) needs to be live before the marketplace is meaningful

## Architectural Sketch

```
bentend daemon
├── Job scheduler (Nomad-style driver plugins)
│   ├── WASM driver (wasmtime)
│   ├── Container driver (containerd + runc)
│   └── VM driver (cloud-hypervisor / firecracker)
├── Resource advertiser (graph Nodes)
├── Bid/ask handler (marketplace subgraphs)
├── Metering + reporting to economic layer
└── Peer transport (iroh)
```

## Known Hard Problems

1. **Container lifecycle does not compose from 12 primitives.** Needs either SANDBOX with rich host functions (architectural risk) or a 13th primitive (breaks the invariant).
2. **Resource verification at scale.** Akash's manual auditor network doesn't scale; their TEE hardware proposal (physical USB dongles) is experimental. Proof of Sampling works for deterministic WASM but not for containers.
3. **Security isolation.** Running arbitrary container workloads on peer hardware is a large attack surface. Firecracker helps; wasmtime helps; generic containers are harder.
4. **Cold start vs always-on tradeoff.** Edge-function-style cold starts are 1-100ms; always-on workloads need persistent resources. The pricing model needs to handle both.

## When to Revive

Conditions for moving this proposal from `future/` to `research/`:
1. Phase 1-3 engine + sync is in production with at least one real community
2. Benten Credits are live and have been used for at least 10,000 non-compute transactions
3. A demonstrated demand from community operators for "rent capacity to my Grove"
4. Someone has solved (or committed to solving) container verification at the economic scale needed

## Source Material

See `docs/research/explore-distributed-compute-vision.md` section 8.6 for the full thinking that produced this proposal.
