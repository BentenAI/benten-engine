# Plugin Manifest

This document describes the **full plugin manifest schema** that lands in Phase 4-Foundation. It is the engineer-facing reference for plugin authors, plugin consumers, and anyone hooking into the engine's plugin lifecycle (install / uninstall / upgrade / share).

> **Disambiguation.** This is the Phase-4-Foundation FULL plugin manifest. See [`MODULE-MANIFEST.md`](MODULE-MANIFEST.md) for the Phase-2b WASM module bundle schema (SANDBOX runtime). The two manifests serve different purposes:
> - **Plugin manifest** (this doc) — describes a *shareable subgraph* of operation Nodes that the engine evaluator walks. The plugin itself runs as code-as-graph; the manifest is the trust envelope.
> - **Module manifest** ([`MODULE-MANIFEST.md`](MODULE-MANIFEST.md)) — describes one or more WASM modules consumed by the SANDBOX primitive at evaluation time. Sandboxed compute, not a shareable application surface.
>
> Cross-reference: a plugin may *contain* SANDBOX nodes whose host-fn invocations point to module-manifest-declared WASM modules. The plugin manifest's `requires` field would name `host:sandbox:exec` cap in that case; the module manifest still lives separately.

---

## §1. Why a plugin manifest exists

A plugin is a content-addressed subgraph (per CLAUDE.md baked-in #18). Without a manifest, two questions are unanswered:

1. **What capabilities does this plugin need to function?** (e.g., "read user's notes", "write to a sandbox label", "use the time host-fn"). The user must consent to the cap envelope at install — they shouldn't be asked on every action.
2. **What can this plugin delegate to other plugins?** A plugin that holds a cap can re-issue UCANs to other plugins inside its own walk. Without policy, an installed plugin could hand any cap it holds to any other plugin. The user's consent at install must bound this.

The manifest answers both as two halves: `requires` (caps the plugin needs) and `shares` (delegation policy). Both halves are signed by the plugin author so they cannot drift post-install.

---

## §2. The four identity concepts

Phase-4-Foundation ratifies a four-distinct-identity-concepts model (CLAUDE.md baked-in #18 "Implementation refinements ratified 2026-05-11" block, retensed post-R1-triage per D-4F-12). NONE of these are conflated in the engine implementation; each lives at a different seam:

| # | Identity | Role | Surface |
|---|---|---|---|
| 1 | **Content-CID** | What the plugin IS — canonical bytes of its subgraph + manifest, content-addressed | `benten-core::Cid`; the engine's content-addressing primitive |
| 2 | **Peer-DID signature on original content** | Provenance — who originally authored or shared the content | `benten-id` peer-DID + Ed25519 signature; `benten-id` RotationLog handles peer-DID rotation/revocation |
| 3 | **Plugin-DID minted at install** | UCAN audience AND constrained issuer within manifest envelope. NOT an attested sub-identity of user-DID. Per D-4F-16: `did:key:...` shape, engine-held Ed25519 keypair via OsRng at install | `benten-id::plugin_did::mint`; persisted in engine's `benten-id` identity store; one keypair per install |
| 4 | **User-DID** | Trust anchor + signs install records + issues UCAN caps with `audience=plugin-DID` | `benten-id` user-DID; root of every cap chain per CLAUDE.md #18 Layer 1 |

**Cross-plugin/schema references use content-CID, not author-DID.** `accepts_content: [hash, ...]` rather than `accepts_author: [did, ...]`. Schema authors can rotate keys without breaking downstream references — the references are CID-keyed.

**Plugin-DID is a UCAN audience AND constrained issuer.** Plugin-DID can issue UCAN delegations to OTHER plugins WITHIN its manifest `shares` policy. Chain validator at `crates/benten-caps/src/manifest_envelope_chain_validation.rs` (G24-D-FP-2) enforces. Plugin-DID has no inherent authority — its issuance is bounded by what the source plugin's manifest allows + the chain must still trace back to a user-DID-issued root grant.

---

## §3. Manifest schema (FULL)

The manifest is a content-addressed DAG-CBOR document. At a minimum it contains:

```
PluginManifest {
  // Identity
  plugin_name: text,                  // Human-readable name (not unique; CID is the unique handle)
  content_cid: Cid,                   // CID of the plugin's subgraph
  peer_did: Did,                      // peer-DID of original author
  peer_signature: Sig,                // peer-DID signature over (content_cid + manifest_body)

  // Capability envelope
  requires: [CapRequirement],         // Caps the plugin needs to function
  shares: SharesPolicy,               // Delegation policy

  // Renderer + composition
  renderer_config: RendererConfig?,   // Optional: output-format + renderer-backend hints
  composes_plugins: [Cid]?,           // Optional: meta-plugin composition (CID-keyed)

  // Cross-references
  accepts_content: [Cid]?,            // Content-CIDs this plugin can consume (e.g., schema CIDs)
  requires_schema_authors: [Did]?,    // Trust-list of peer-DIDs for schemas this plugin reads
  requires_plugin_authors: [Did]?,    // Trust-list of peer-DIDs for plugins this plugin composes
}
```

At install time, the user-DID signs an `InstallRecord` referencing this manifest:

```
InstallRecord {
  manifest_cid: Cid,
  consenting_user_did: Did,
  user_signature: Sig,                // user-DID signature over (manifest_cid + timestamp + nonce)
  timestamp: Hlc,
  nonce: Bytes,
  granted_caps: [CapGrant],           // UCAN delegations from user-DID to plugin-DID
}
```

The install record (NOT the manifest) carries the user's consent. The manifest is what the plugin author published; the install record is what the user agreed to.

### §3.1 `requires` shape

A `CapRequirement` names a typed scope the plugin needs to call upon. Phase-4-Foundation `manifest_scope.rs` maps to:

- `requires:<plugin_did>:<requirement_path>` — engine cap-scope shape (canonical; see §6 scope grammar)
- typed-by-domain (e.g., `store:notes:read` / `host:time:now` / `host:sandbox:exec`)

If a plugin's requires include `host:sandbox:exec` AND the installing peer is a thin-compute-surface (browser / edge per CLAUDE.md #17), install fails with `E_PLUGIN_HETEROGENEITY_INCOMPATIBLE` (heterogeneity contract).

### §3.2 `shares` shape

`SharesPolicy` is the delegation envelope:

```
SharesPolicy {
  default: SharesPolicyDefault,       // "none" | "any" | "matching"
  rules: [SharesRule]?,               // Per-cap or per-plugin-target rules
}

SharesRule {
  cap_pattern: text,                  // e.g., "store:notes:read"
  target: SharesTarget,               // "any" | { plugin_did: Did } | { plugin_author: Did }
}
```

Conservative default for v0 plugins: `shares: { default: none }`. Phase 4 ecosystem broadens cautiously.

**Private-namespace data residency** is automatic: any cap-scope of shape `private:<plugin_did>:*` is implicitly `shares: none` regardless of declared policy. `crates/benten-caps/src/private_namespace_policy.rs` enforces.

---

## §4. Install / uninstall / upgrade flow

### §4.1 Install

1. Receiver peer receives plugin bytes (out-of-band content-addressed-share over Atriums in Phase 4-Foundation; decentralized registry → Phase 4-Meta per post-R1-triage ratification #3).
2. Receiver verifies: (a) bytes hash to declared content-CID; (b) peer-DID signature on content; (c) peer-DID is in user's `requires_plugin_authors` trust-list (else first-install prompt).
3. Engine mints fresh plugin-DID keypair via OsRng at `benten-id::plugin_did::mint`.
4. Admin UI surfaces manifest review screen (per `docs/ADMIN-UI.md`): plain English requires/shares, per-cap-grant decline option.
5. On consent, user-DID signs `InstallRecord` + issues UCAN delegations from user-DID to plugin-DID for granted caps.
6. Install record persisted to `ManifestStore` (redb backend, shares storage with GrantStore per cap-r1-15).
7. Plugin enters user's "plugin library" subgraph (per D-4F-14 — content-addressed; cheap; holds all installed versions + forks).
8. Active reference (per ratification #2 — Loro Map per-device-keyed CURRENT) updates to point at this plugin-version.

Meta-plugin composition cycle detection runs at install (per post-R1-triage Q2 ratification): rejects with `E_PLUGIN_META_COMPOSITION_CYCLE_REJECTED` if the install would create a cycle in the `composes_plugins` graph.

### §4.2 Uninstall

The G24-D-FP-1 `uninstall_plugin` seam (forthcoming in `crates/benten-platform-foundation`):

1. Enumerate all user-DID-issued grants WHERE `audience=plugin-DID`; revoke each.
2. Cascade plugin-DID's own downstream UCAN delegations (enumerate grants issued BY plugin-DID; revoke each).
3. Terminate all live subscriptions whose subscriber DID was this plugin.
4. Delete private namespace data (`private:<plugin_did>:*` rows).
5. Remove from manifest store.
6. Remove library entry (update plugin library subgraph + active reference per ratification #2).
7. Emit `PluginUninstalled` change-event.

### §4.3 Upgrade

`upgrade(plugin_did, new_content_cid)` (per post-R1-triage ratification #8 cap-change-triggered fresh consent):

1. Verify peer-DID signature on new content matches the previously-installed peer-DID (peer-DID change at upgrade = re-install, user re-consents per T10-upgrade).
2. Verify DAG-shaped version chain monotonicity: new CID is a descendant of installed in the DAG (per D-4F-14 anchor + Version Node DAG extension). Older-version "upgrades" rejected.
3. **Cap-change consent rule** (ratification #8):
   - Silent within-lineage upgrade if `requires` is a strict subset of installed manifest;
   - Full re-consent if `requires` GREW (any cap added or scope widened);
   - Cross-fork merge = user-initiated through same consent flow (no separate cross-fork merge surface).
4. Atomic transaction: old reference dropped + new reference in plugin library + active-ref update in one commit.

No manifest-schema-version check (per D-4F-13 — CID covers shape; pull-not-push obviates schema-version field).

### §4.4 Share

Direct content-addressed-share over Atriums (Phase 4-Foundation v0 scope per ratification #3). Receiver verifies content-CID + peer-DID signature; install flow above runs. Decentralized self-discovered registry is Phase 4-Meta scope.

---

## §5. DAG version chains

Per D-4F-14, version chains extend the Phase-1 `Anchor + Version Node + CURRENT pointer` pattern to **DAG-shape** (branches + forks):

```
   anchor
     │
     v1
     │
     v2-mainline
    / \
   v3  v2.5-fork
    \   │
     \  v3-fork
      \ /
       v4-merge (user-initiated cross-fork merge per ratification #8)
```

CURRENT pointer can point at any reachable branch tip. Per-device-local CURRENT (ratification #2 — Loro Map per-device-keyed); user "switches active version" = updates the local CURRENT pointer; sync surface presents per-device-keyed map.

The `version_chain.rs` extension is in `crates/benten-core`. New primitive mints? NO — uses existing 12-primitive vocabulary (NEXT / FORK_OF / CURRENT pointer; no new edge-label kinds minted per cag-r1-4).

---

## §6. Cap-scope grammar

Canonical scope-string shapes (per plugin-arch-r1-10 + sec-4f-r1-7):

- `private:<plugin_did>:*` — private namespace; `shares: none` always; cap-policy rejects cross-plugin delegation
- `requires:<plugin_did>:<requirement_path>` — cap-scope derived from manifest `requires` half
- `shares:<plugin_did>:<share_path>` — cap-scope derived from manifest `shares` half

`crates/benten-caps/src/manifest_scope.rs` (G27-D) implements the mapping. `NoAuthBackend` defaults permit even for plugin-issued scopes (matches CLAUDE.md baked-in #7 pluggable-policy default; explicit doc-comment in code).

`GrantBackedPolicy::derive_write_scope` (G27-B) and `GrantBackedPolicy::check_read` (G22-FP-3 with `ctx.actor_cid` consultation) both consult manifest scope.

---

## §7. Renderer config (optional)

`RendererConfig` disambiguates manifest envelope (CAPS) vs renderer-config (rendering targets / UI hosting / bundle layout) per plugin-arch-r1-15:

```
RendererConfig {
  output_format: OutputFormat,        // "html_json" | "plaintext" | (others)
  renderer_backends: [RendererBackend]?,
  hosting_target: HostingTarget?,     // "browser_wasm32" | "tauri_embedded_webview"
  bundle_size_budget_kb: u32?,
}
```

Optional because not every plugin renders user-facing output (a SUBSCRIBE-only plugin might have no rendering). Admin UI v0 fills this with `output_format: html_json`, `renderer_backends: [browser_wasm32, tauri_embedded_webview]`.

---

## §8. Trust model summary

Per CLAUDE.md baked-in #18:

- **Layer 1 user-as-root.** Every cap chain traces back to a user-issued root grant. No plugin gets capability without user consent somewhere in the chain.
- **Layer 2 install-time manifest envelope.** User reviews `requires` + `shares` at install; consents to envelope; both halves signed by plugin author so can't drift post-install.
- **Layer 3 runtime delegation within envelope.** Plugins delegate UCANs to each other freely *if* the request fits source plugin's manifest `shares`. `CapabilityPolicy` validates chain at access-time: chain traces to user-root + each delegation step fits source plugin's policy + requested cap within attenuation envelope.

ErrorCodes (Phase 4-Foundation mints): `E_PLUGIN_MANIFEST_INVALID` / `E_PLUGIN_INSTALL_RECORD_USER_SIGNATURE_INVALID` / `E_PLUGIN_CONTENT_PEER_SIGNATURE_INVALID` / `E_PLUGIN_CONTENT_PEER_KEY_ROTATED` / `E_PLUGIN_AUTHOR_NOT_TRUSTED` / `E_PLUGIN_INSTALL_CONSENT_REQUIRED` / `E_PLUGIN_DELEGATION_OUTSIDE_MANIFEST_ENVELOPE` / `E_PLUGIN_PRIVATE_NAMESPACE_DELEGATION_FORBIDDEN` / `E_PLUGIN_CONTENT_CID_MISMATCH` / `E_PLUGIN_NEW_VERSION_AVAILABLE` / `E_PLUGIN_HETEROGENEITY_INCOMPATIBLE` / `E_PLUGIN_META_COMPOSITION_CYCLE_REJECTED`. See [`ERROR-CATALOG.md`](ERROR-CATALOG.md) for the full catalogue.

---

## §9. Cross-references

- **CLAUDE.md baked-in #18** — three-layer consent model (canonical)
- **CLAUDE.md baked-in #19** — engine-level extensions are Rust crates (distinct trust model; plugins are NOT engine extensions)
- [`ARCHITECTURE.md`](ARCHITECTURE.md) §"Plugins and engine extensions" — workspace shape + crate boundaries
- [`SECURITY-POSTURE.md`](SECURITY-POSTURE.md) "Plugin trust model" — security narrative
- [`MODULE-MANIFEST.md`](MODULE-MANIFEST.md) — Phase-2b WASM module manifest (distinct surface)
- [`SCHEMA-DRIVEN-RENDERING.md`](SCHEMA-DRIVEN-RENDERING.md) — schema vocabulary; plugin manifests reference schema CIDs
- [`ADMIN-UI.md`](ADMIN-UI.md) — admin UI v0; first plugin instance + manifest review UX

---

(Phase-4-Foundation companion doc lands at G24-D canary per `feedback_post_fix_doc_coupling_preflight.md` §3.5b HARDENED + meth-r1-7 companion-with-canary discipline.)
