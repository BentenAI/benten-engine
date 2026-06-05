# Compute marketplace + economics (Phase-5+; PHASE-LATER-DEFER)

This document is the destination for the **Phase-5+ compute / economics** surfaces (R0.7 §2.8 CE-1 + §4.3). The
load-bearing v1-beta property: **the compute / economic surface adds ZERO frozen wire field** — economics
COMPOSES from already-frozen primitives + graph-native Nodes, so nothing here gates the G-CORE-9 v1-interface
freeze. The wire types named below are **PHASE-LATER-DEFER**, registered as **V1-FROZEN Rows D-28 / D-29** in
`docs/V1-FROZEN-INTERFACE-DEFERRED.md`.

---

## §2.8 — Compute (CE-1; first-class fungible per Ben)

When the compute marketplace activates (Phase-5+), it is **graph-native**:

- **`PeerResource` graph Nodes** advertise a peer's offered resources, typed by
  **`ResourceKind { Compute | Storage | Bandwidth | Availability }`**.
- **`OwnerRef { Member | Community | ThirdParty }`** names who owns / offers the resource.
- **Economics COMPOSES** from three already-existing primitives — **{ UCAN caveats + a Credits-ledger-as-graph +
  a signed `CommunityEconomicPolicy` Node }** — with **ZERO MembershipSet wire field**. There is no new frozen
  codepoint, no new envelope, no MembershipSet hook.
- **"local-free" is a default `local_rate` policy-field**, NOT a hard engine rule — a community can set its own
  `local_rate` in the signed `CommunityEconomicPolicy` Node; the engine does not bake a free-tier rule into the
  wire.

This is why the **`economic_policy` reserve on `MembershipSetPolicy` is DROPPED** (CE-1 H2): economics composes
from the primitives above; there is **ZERO economic freeze hook** on the MembershipSet. (The F-FREEZE-1 catch-net
asserts `economic_policy` is absent from the source tree + the compute wire types are absent from the frozen wire
at v1-beta.)

---

## §4.3 — Surface dispositions

| Surface | Disposition | Notes |
|---|---|---|
| `PeerResource` / `ResourceKind` / `OwnerRef` / `CommunityEconomicPolicy` | **PHASE-LATER-DEFER** | V1-FROZEN **Rows D-28 / D-29** (minted at the doc-wave; in-tree max was D-27); GRAPH-NATIVE when activated |
| `economic_policy: Option<opaque>` reserve on `MembershipSetPolicy` | **DROPPED** | CE-1 H2; economics composes; ZERO freeze hook |
| Garden / Grove governance impl (voting / moderation) | **PHASE-LATER-DEFER** | Phase-4-Meta-Composing+ (GovernanceConfig signed-Node content) |
| Federation recursion impl (Model-A opt-in) | **PHASE-LATER-DEFER** | post-v1-beta (Inv-20 k recursion-bound) |

---

## Why defer (and why it's free to defer)

The compute marketplace is deferred to Phase-5+ because (a) it is not v1-beta-gating (it adds no frozen wire), and
(b) it is **graph-native when activated** — `PeerResource` / `CommunityEconomicPolicy` are ordinary signed graph
Nodes, so activating the marketplace is an additive application-layer build on the frozen v1 substrate, never a
wire-format break. The single destination doc for the deferred wire surfaces is
`docs/V1-FROZEN-INTERFACE-DEFERRED.md` (the global `D-NN` namespace) — Rows **D-28** (compute-marketplace wire
types) + **D-29** (economic-policy composition surface).

---

## Cross-references

- `docs/V1-FROZEN-INTERFACE-DEFERRED.md` — Rows D-28 / D-29 (the deferred wire-surface registry).
- `docs/CRYPTO-CODEPOINTS.md` — §4.4 net-frozen-surface tally (CE-1 drops the economic reserve; ZERO hook).
- `docs/FULL-ROADMAP.md` — Phase-5+ scope (the compute / economics pillar).
