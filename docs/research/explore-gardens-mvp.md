# Exploration: Digital Gardens MVP (Phase 7 scoping)

**Created:** 2026-04-14
**Status:** ACTIVE RESEARCH. Scoping doc for Phase 7 committed work. Full spec emerges during Phase 7 pre-work (ADDL process).
**Audience:** Anyone planning Phase 7, or writing specs for the community layer.

---

## Framing

Atriums ship in Phase 3 (P2P sync between trusted peers). Gardens MVP ships in Phase 7 — community spaces beyond direct Atrium sharing. Full Groves (fractal/polycentric governance) remain exploratory.

The Gardens MVP is the minimum viable community layer that makes the social vision real without committing to the full governance complexity. It's what a small team or a friend group actually needs.

## Atrium → Garden Distinction

| Feature | Atrium (Phase 3) | Garden MVP (Phase 7) |
|---------|------------------|----------------------|
| Membership | Direct peer-to-peer grants | Admin-managed; invitation flow |
| Governance | None (consensus by convention) | Admin-configured rules |
| Content policies | None (peers share everything they agree to) | Configurable (what's postable, moderation) |
| Moderation | None (mutual block only) | Basic (admin can remove content / members) |
| Size | Small (2-20 people) | Medium (20-500 people) |
| Invitation | Out-of-band | Named invitation Nodes |
| Name/identity | Informal | A Garden has a name, description, membership list as first-class Nodes |

An Atrium promotes to a Garden when it needs structure: more members, more rules, more moderation. Promotion is a configuration change, not a data migration — the same subgraphs apply; a GovernanceConfig Node and admin grants get added.

## What Ships in Phase 7

### 1. Garden Creation Flow

- Promote an existing Atrium to a Garden (preserves history)
- Create a new Garden from scratch
- Garden metadata (name, description, topics, visibility: public discovery vs. invite-only)
- Initial admin assignment (creator is admin by default)

### 2. Invitation Flow

- Admin creates invitation Nodes with expiry + scope
- Invitation conveys: "join this Garden as [member | moderator | admin]"
- Invitee accepts via their own instance; their instance begins syncing the Garden's subgraph
- Invitation can be link-based (shareable URL) or direct (capability grant to a known DID)

### 3. Admin-Configured Governance

Unlike Groves (which have fractal/polycentric/configurable voting), Gardens have simple admin governance:
- Admins can change content policies
- Admins can appoint moderators (subset of admin powers)
- Admins can remove members (with configurable grace period before data stops syncing)
- Admins can remove content (creates "removed" tombstones; original preserved in history for audit)
- Member role hierarchy: admin > moderator > member > invitee

**Key constraint:** admin power is within the Garden only. An admin cannot reach outside the Garden's subgraph scope.

### 4. Content Policies

- What content types members can create
- What content requires approval before publishing
- What capabilities members inherit on join
- Rate limits (anti-spam)

### 5. Member-Mesh Replication

- Every member's instance holds a copy of the Garden's graph (within sync scope)
- New-member bootstrap via Merkle diff from any online member (parallel peer serving)
- Gossip for propagation (GossipSub with fanout = 6, adequate for ~1000-member Gardens)
- Tombstone garbage collection per existing sync spec (90-day default)

### 6. Basic Moderation Tooling

- Content removal (tombstone the content Node; original visible in history for admins)
- Member muting (member can still read, can't write)
- Member banning (member's sync agreement terminates; their device no longer syncs this Garden)
- Reporting flow (members flag content; admins/moderators review via an IVM-materialized queue)
- Moderation actions are logged as graph Nodes (auditable)

## What Does NOT Ship in Phase 7

Reserved for full Groves (exploratory):
- Configurable voting mechanisms (1p1v, quadratic, conviction, liquid delegation)
- Fractal governance (sub-communities with inherited/overridden rules)
- Polycentric federation (multiple parent communities)
- Formal meta-governance (changing how governance changes)
- Knowledge attestation marketplace
- Fork-and-compete dynamics at the Grove level (fork-is-a-right is still guaranteed at the engine level)

## Integration With Other Phases

- **Phase 3 Atrium sync** — Gardens are Atriums with extra rules. Reuse all sync infrastructure.
- **Phase 4 Thrum migration** — Thrum modules (blog, wiki, forum) become per-Garden installable content types.
- **Phase 5 platform features** — schema-driven rendering means Gardens automatically get UI for new content types.
- **Phase 6 AI Assistant** — the assistant can participate in a Garden (browse, post with user approval, help members organize). Multi-user Garden dynamics for the AI assistant need thought.
- **Phase 8 Credits** — Gardens can have treasuries, member fees, or paid features. Basic for MVP; full economics in Groves.

## Design Questions for Phase 7 Pre-Work

1. **Default visibility.** Private-by-default (invite-only) or public-by-default (discoverable)? Matters for network effects vs. privacy.

2. **Discovery mechanism.** Are there public Gardens browseable by everyone? Through what index? Decentralized directory?

3. **Moderation transparency.** Full public audit log vs. admin-private? Mismatch with "fork is a right" if moderation is opaque.

4. **Promotion from Atrium.** Requires unanimous Atrium consent, or just the creator's decision? What if some Atrium members don't want to be in a Garden?

5. **Garden → Grove promotion.** When a Garden outgrows simple admin governance, how does it promote to a Grove? (Out of Phase 7 scope, but the data model should support this evolution cleanly.)

6. **Member storage burden.** A 500-member Garden with active content could be GBs. How do we handle members with limited storage? Lazy sync? Opt-in replication levels?

7. **Invitation spam.** Link-based invitations are useful but spammable. Rate limits? Invitation chains (Alice invited Bob who invited Carol)?

8. **Cross-Garden identity.** A user is in 20 Gardens. Do they have a unified profile across all of them, or per-Garden identity?

## Competitive Positioning

Gardens MVP competes most directly with:
- **Discord servers** — real-time + threaded, moderated by roles. Our advantage: user-owned data, no central server, rich content types via Thrum.
- **Slack workspaces** — team-oriented, integration-heavy. Our advantage: self-owned, no subscription, AI-native integration via Phase 6 assistant.
- **Matrix homeservers** — decentralized but complex. Our advantage: no homeserver operation burden (member-mesh).
- **Mastodon instances** — decentralized but fediverse-style federation. Our advantage: Gardens are explicit groups, not an open-by-default fediverse.

## When Full Groves Earn Committed

Phase 7 ships Gardens MVP. Groves (exploratory) require:
- Gardens in active production use for 6+ months
- Real demand from Gardens that outgrow admin governance
- A governance design that handles sybil attacks, fork-bomb confusion, polycentric authority injection (research exists; needs implementation)
- An owner for the Groves scope

## Source Material

- [`docs/research/explore-fractal-governance.md`](explore-fractal-governance.md) — the full Groves research (kept active)
- [`docs/archive/v1-critiques/critique-mesh-sync.md`](../archive/v1-critiques/critique-mesh-sync.md) — member-mesh architectural concerns
- [`docs/archive/v2-reviews/v2-review-p2p.md`](../archive/v2-reviews/v2-review-p2p.md) — sync protocol findings
- Matrix room governance, Discord moderation patterns, Scuttlebutt pub model for precedent
