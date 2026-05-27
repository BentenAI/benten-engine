# Option F+ §6.2 envelope-layer-unification — LENS L11: CRDT / EVENTUAL-CONSISTENCY / CONFLICT-RESOLUTION / OFFLINE-FIRST

**Branch:** `phase-4-meta-core/option-f-plus-lens-l11-crdt-conflict-resolution`
**Role:** Senior distributed-systems architect filling a coverage gap C4 process-discipline flagged: how does the F-full encryption substrate compose with Benten's eventual-consistency CRDT layer when conflicting envelopes target the same Node-CID, when peers fork, when offline-replay collides with replay-window enforcement, and when concurrent `ExecuteWorkflow` invocations write conflicting results.
**Date:** 2026-05-27
**Tree-state pre-flight:** Started clean at `2172cb6d` on isolated worktree.
**Brief lineage:** Distinct from L9 (Atrium-integration P2P architect) — L9 explicitly notes "Multiple Drops targeting the same Node-CID = standard CRDT merge case … §6.2 envelope is transparent to this — it carries the payload; CRDT merge is downstream of envelope-Open" and parks the question. This lens takes the parked question seriously and asks where exactly the seam breaks.

---

## §1 Executive verdict + confidence

### Top-line

The F-full substrate composes **acceptably but not seamlessly** with Benten's existing per-property HLC-LWW Loro CRDT layer (`benten-sync/src/crdt.rs`). The dual-CID model (U18) + recipient-key-generation (U19) + K_principal-generation (U20) amendments — added by L9 specifically to handle three CRDT-encryption interaction classes — close the **single-writer-per-Node** cases. They do **not** close the **multi-writer-concurrent-encrypt** cases, and they introduce four **new** CRDT hazards the panel did not name:

1. **K(N) keying instability under CRDT merge** — Layer-B's `K(N) = KDF(K_principal, N.cid)` is a function of Node-CID, but Node-CID changes when CRDT merge produces a new Version Node (D-C HYBRID per `arch-r1-4`). The encrypted blob under the old K(N) becomes unindexed after merge; the new merged-Node needs a fresh AEAD pass under a fresh K(merged). No amendment specifies who performs that re-encryption or how the old encrypted bytes are GC'd. **HIGH severity.** This is the load-bearing finding.
2. **K_principal-generation rotation log is itself a CRDT-mutated Atrium Node** (per U20: "Mint `KPrincipalRotation { generation, rotated_at, rotated_by_device_did, reason }` Atrium-replicated Node"). Two devices rotating concurrently produce divergent generation counters. The amendment is silent on the conflict-resolution rule. C4 flagged this gap; the registry has not closed it. **HIGH severity.**
3. **Replay-window (U5/U28) vs offline-resync race** — a peer offline ≥`valid_until` legitimately receives DeviceLink + RemotePermission envelopes that were sealed before the deadline. Strict `now() > valid_until` rejection drops them; loose acceptance defeats the replay defense. The amendment registry does not name a resolution. **MEDIUM-HIGH severity** for DeviceLink + RemotePermission (Drop bundles + Vault are correctly excluded per L9 + L3).
4. **Per-stanza Drop bundle + per-Node-AEAD CRDT-merge state mismatch** — when a Drop bundle carries `Vec<NodeUpdate>` to recipient and Bob's local CRDT already advanced past one of those Nodes, Bob's merged-Node now has a CID that doesn't match the Drop's `plaintext_cid` for that Node — the multi-stanza AAD-cross-reference defense (U17) flags it as a substitution attack rather than as expected divergence. **MEDIUM severity.**

### Confidence

**MED-HIGH that all four CRDT-encryption gaps are real.** Verified against `crates/benten-sync/src/crdt.rs` per-property HLC-LWW design + D-C HYBRID merge-to-new-Version-Node pattern. The K(N) re-keying break (Gap-1) is **HIGH confidence** because Layer-B's CID-derived keying is structural and Node-CID-changes-on-merge is structural. Gap-2 is HIGH because the rotation-log is named Atrium-replicated by U20 itself. Gap-3 is HIGH because U5 explicitly excludes Drops+Vault but does not exclude offline-replay. Gap-4 is MED because it depends on Drop-bundle granularity choices not yet pinned at R0.

**MED confidence that the recommended additional amendments below are sufficient.** Three of the four gaps have well-precedented solutions (re-key-on-merge per Yjs/Automerge encrypted-CRDT, generation-vector for rotation logs per Signal/MLS, offline-replay carve-out per UCAN/Macaroons). The fourth (Gap-4) is genuinely novel and needs more design at R0 §4.

### Recommendation envelope

Add **four new amendments** to the 28-amendment registry:
- **U41 (LOAD-BEARING wire-affecting)** — CRDT-merge re-encryption discipline + AAD-binding extension. ~3-4 wave-days.
- **U42 (LOAD-BEARING wire-affecting)** — K_principal-rotation-log CRDT conflict-resolution rule + `k_principal_generation` as a multi-writer vector clock, not a scalar. ~2-3 wave-days.
- **U43 (LOAD-BEARING wire-affecting)** — replay-window offline-replay carve-out via `received_at_epoch` + `outbox_retransmit_seq`. ~1-2 wave-days.
- **U44 (RECOMMENDED)** — Drop-bundle Node-update granularity discipline + per-Node `expected_predecessor_cid` chain to disambiguate substitution-attack vs CRDT-merge-divergence. ~2-3 wave-days.

**Total CRDT-lens incremental cost: ~8-12 wave-days = ~1.5-2.5 calendar-weeks.** Fits inside the existing ~13-15 week v1-beta envelope per consolidator §6, raising the high-end to ~14-17 weeks (still within the 7-15 stretch with a small overrun risk).

### What this lens does NOT cover

- Pure CRDT correctness independent of encryption — `benten-sync/src/crdt.rs` already pins `prop_loro_concurrent_writes_converge_via_hlc_ordering`; out of scope here.
- Atrium peer-discovery / iroh-blobs replication / sendme transport — L9 covers, MEDIUM confidence per L9 §5.4.
- Formal-methods tractability of CRDT-encrypted-merge under adversarial model — C5 formal-methods lens scope.

### Layer-of-abstraction stance (Task 6 preview)

Of the three options (envelope-layer amendments / application-layer CRDT-handles-everything / Atrium-coordination-layer version-vectors): the **honest answer is (a)-with-(c)-support**. The envelope-format MUST carry enough binding information to make CRDT-merge unambiguous (Gap-1 + Gap-4); the Atrium layer MUST coordinate generation-vector convergence for K_principal-rotation (Gap-2); the application layer can stay below the seam if the envelope does its job. The "envelopes are opaque transport" stance fails because U4 + U5 + U17 already put sender-DID + epochs + per-stanza-AAD into the envelope — the envelope is **not** opaque to application semantics today, and we should stop pretending it is.

---

## §2 CRDT-encryption interaction scenarios (Task 1)

### Scenario A — Alice updates Node-X concurrently on device-1 + device-2

**Setup.** Alice has Node-X with CID-X. Alice's device-1 (offline) edits the property `title` → "Hello". Alice's device-2 (offline) edits property `body` → "World". Both produce divergent local Nodes. Devices come online + Atrium sync delivers each device's update to the other.

**What §6.2 + amendments specify.**
- Each device encrypts its local Node-X' under `K(N) = KDF(K_principal, N.cid)` where N.cid is the **local** post-update CID. So device-1 has CID-X1' encrypted under K(X1'); device-2 has CID-X2' encrypted under K(X2').
- Atrium replicates both encrypted blobs.
- On merge, `benten-sync/src/crdt.rs` per-property HLC-LWW resolves the concurrent writes via HLC ordering. Per the D-C HYBRID pattern (`arch-r1-4`), the merged Node is **encoded as a new Version Node minted via Anchor + Version + CURRENT pattern**.

**Where it breaks.**
1. **The merged-Node has a fresh CID-X3** (combining title from one and body from the other). K(X3) is a fresh per-Node AEAD key the engine has never derived.
2. **Neither device-1's nor device-2's encrypted blob decrypts to the merged plaintext.** Both decrypt to their own divergent versions.
3. **Question: who re-encrypts the merged plaintext under K(X3)?** The merge happens at the engine layer with K_principal in scope, so the engine CAN re-encrypt. But:
   - The merger device might be neither device-1 nor device-2 (e.g. device-3 received both blobs and merges). Device-3 must derive K(X3) + re-encrypt.
   - If the merge happens on a relay-peer that doesn't have K_principal (e.g. an iroh-blobs storage node), it CANNOT merge — only deliver both blobs and let an authorized device merge.
   - U18 dual-CID specifies `plaintext_cid` (graph-stable) vs `envelope_blob_cid` (transport-mutable), but **does not specify what `plaintext_cid` means after CRDT merge**. Is `plaintext_cid` the CID of the merged plaintext (X3)? Or the CID of the pre-merge divergent plaintext that this specific envelope carries?
4. **The encrypted blobs under K(X1') and K(X2') become unreferenced after merge.** Layer-B GC discipline must reap them. No amendment specifies this. If we keep them forever, every CRDT-merge cycle doubles storage; if we reap immediately, we lose the ability to replay history.

**Convergence outcome.** Plaintext-CRDT-side: converges via HLC-LWW (engine handles). Encrypted-bytes-side: **diverges permanently** unless the engine performs explicit re-encryption AND publishes the new encrypted bytes back to Atrium. None of U17/U18/U19/U20 explicitly mandate this; it's implicit in "engine re-encrypts after merge", but the amendment registry doesn't pin the discipline.

**Verdict: BREAKS without new amendment.** Add U41 (CRDT-merge re-encryption discipline).

### Scenario B — Alice + Bob collaborative edit on the same Node-X

**Setup.** Alice + Bob both have read+write on Node-X via UCAN/capability grants. Both make concurrent edits. Bob's edit is encrypted under Bob's K_principal-derived K(X) — wait. Per Layer-B, K(N) is keyed by K_principal which is **per-principal**. Alice's K(X) ≠ Bob's K(X). So Alice encrypts under K_alice(X), Bob encrypts under K_bob(X).

**What §6.2 + amendments specify.**
- Per F-full Layer-C: collaborative sharing happens via Drop bundles. Alice would send a Drop containing K_alice(X)-encrypted Node, sealed under Bob's HPKE-pubkey. Bob's device unwraps, gets Node plaintext, can then re-encrypt under K_bob(X) for Bob's own at-rest storage.
- For collaborative edit: this means every edit by Bob requires a Drop-bundle round-trip back to Alice with re-encryption under K_alice(X).
- **OR** Alice + Bob share a group-K(X) (CGKA-derived) — but CGKA is deferred to post-v1-beta per consolidator §3 #35 + #42.

**Where it breaks.**
1. **Without group-CGKA, collaborative edit is "Drop-bundle ping-pong"** — every co-editor must re-Drop on every edit. This is structurally non-converging in any latency-tolerant offline-first sense.
2. **Even with Drop-bundle ping-pong, the merge question of Scenario A reappears at the Drop-bundle layer.** Two Drops arrive concurrently at Alice's device with conflicting Node-X plaintext. Alice's engine HLC-LWW-merges, produces merged Node-X3, must re-encrypt + re-Drop to Bob.
3. **Recipient-side authorization conflict.** Drop-A says "Bob has read until T_1"; Drop-B says "Bob has read until T_2". L9 §5.1 says "UCAN scope semantics make these additive (Bob holds both grants; effective access is the union)." Fine for read-grants; **catastrophic for revocations** (Drop-A says "Bob has read until T_1"; Drop-C is a revocation at T_0.5; if the CRDT merges via "union", Bob retains read after revocation).

**Verdict: BREAKS for true collaborative edit; ACCEPTABLE for review-after-edit with explicit re-Drop.** Document the limitation in `docs/SECURITY-POSTURE.md`: **"collaborative edit at v1-beta requires re-Drop per edit; CGKA-based group-K for true concurrent multi-writer Node is deferred to post-v1-beta per Compromise #42."** This is an amendment to U17 + a new Compromise #45 (collaborative-edit-via-re-drop accepted-trade-off).

### Scenario C — Alice shares Node-X with Bob; then Alice updates Node-X locally

**Setup.** Alice sends Bob a Drop bundle for Node-X with `plaintext_cid = CID-X` + `envelope_blob_cid = CID-blob1`. Bob's local CRDT now contains Alice's Node-X view. Alice then locally updates Node-X to X' (new local CID-X').

**What §6.2 + amendments specify.**
- The previously-shared Drop bundle remains valid (per U18: `plaintext_cid` stable; `envelope_blob_cid` references original blob; both still exist on iroh-blobs).
- Bob is **NOT** automatically notified. There is no push semantic in §6.2 — Drops are pull-on-demand from Atrium routing.
- U19 specifies recipient-key-rotation generation tracking; this is symmetric for **sender-side updates**: there is no `sender_content_generation` in BindingContext::DropToRecipient. So Alice's content-version-2 has no wire-binding to its predecessor.

**Where it breaks.**
1. **Bob's local CRDT thinks Node-X is at CID-X.** Bob does not know X' exists.
2. **If Alice's update is conceptually a "new revision pointing back to X" (`previous_version_cid: CID-X`),** the version-chain primitive captures the relationship; but the Drop bundle for X' is a **separate envelope-blob** with no wire-pointer to the prior CID-blob1. Bob receives only X' Drop; his CRDT-merge must figure out that X' supersedes X via inspecting `previous_version_cid` inside plaintext after Open.
3. **Forkability composition.** If Alice's Atrium has forked between the X-Drop and the X'-Drop, the X'-Drop's audience set may differ from the X-Drop's. Bob in the post-fork branch may or may not receive X'. The dual-CID model preserves the X-Drop's validity (good); but Bob's CRDT may end up with X as "current" forever because no X' arrives. This is exactly the documented forkability semantic ("member-leaves-keeps-past-content"); but Bob can't tell whether he's missing X' because of fork-exclusion or because of network partition.
4. **Replay attack.** An adversary holding Alice's Drop bundle for X' can deliver it to Bob arbitrarily late. Bob's CRDT correctly orders via HLC; but the Drop bundle's `sealed_at_epoch` (per U5 — wait, U5 explicitly excludes DropToRecipient from time-bound). So there's no in-envelope replay-window. The only freshness signal is the inner content's HLC. This is acceptable per Compromise #31's "forever-valid drops" stance.

**Verdict: WORKS with documented expectations.** Bob's CRDT-merge resolves correctly via HLC. The "Bob doesn't know he's missing X'" issue is fundamentally a sync-coverage problem, not an encryption problem. No new amendment needed; document in `docs/ARCHITECTURE.md` Drop-bundle-update-semantics section.

### Scenario D — Alice offline 6 months; Bob sends 100 Drops; replay-window collision

**Setup.** Alice's device offline T₀ to T₀+6 months. Bob sends 100 Drop bundles during the offline period, each `sealed_at = T₀ + k·hours`, each with `valid_until = sealed_at + 1 day` (per U5's example default). Alice comes back at T₀+6 months. Atrium sync delivers all 100.

**What §6.2 + amendments specify.**
- U5 explicitly EXCLUDES `DropToRecipient` from time-bound binding ("vault is at-rest by design; drops are intentionally long-lived per Compromise #31"). So Drop bundles have NO `valid_until`; all 100 are accepted regardless of clock.
- L9 §5.1 explicitly confirms: "Composes cleanly with eventual-consistency CRDT semantics with NO §6.2 changes."

**Where it breaks (THE NEW FINDING).**
1. **L9 + U5 cover Drop bundles correctly.** Drops are NOT bound by replay-window. ✓
2. **But DeviceLink + RemotePermission + RemotePermission-ExecuteWorkflow (U21) ARE bound by U5's replay-window.** Bob is the user's friend and was setting up a remote-execute permission grant for Alice's compute peer at T₀ + 3 months with `valid_until = T₀ + 4 months`. Alice's compute peer is offline; at T₀ + 7 months it comes back online and receives the grant. The grant's `valid_until` has expired by 3 months. U5 mandates structural rejection.
3. **Is this the right behavior?** YES for security (an expired grant should not be honored). But:
   - The compute peer cannot distinguish "I was offline when this was issued and the grant was honest but is now stale" from "an adversary is replaying a leaked grant".
   - The user's mental model: "I authorized Alice's compute peer to do X for one month, three months ago; if it comes back online now, the authorization should NOT activate." This matches U5.
   - But there is a **second-order effect**: any RemotePermission grant that depends on prior RemotePermission deliveries (chained grants) breaks if any intermediate grant is replay-rejected. Example: Alice grants Bob → Bob delegates to Carol (chain via UCAN). If Bob's delegation-Drop arrives at Carol after Bob's own grant has been replay-rejected by Carol's clock, Carol has no way to honor the chain.
4. **Real fix.** The replay-window enforcement should be at the **point of first-receipt** (`received_at_epoch <= valid_until`), not point-of-use (`now() <= valid_until`). This requires adding `received_at_epoch` as a recipient-bound binding (NOT in the AAD because the sender doesn't know it; the recipient stamps it on receipt + persists). The U5 check becomes `received_at_epoch <= valid_until` once the recipient has accepted; subsequent uses inside the validity window proceed as normal. Offline-receipt-after-validity-expiry: receipt-time outside window = structural reject (correct for security).
5. **Compromise route.** Sender may explicitly mark a grant as "offline-tolerant up to X months" via `offline_acceptance_grace_seconds: u32`; the recipient accepts if `received_at_epoch <= valid_until + offline_acceptance_grace_seconds`. This is opt-in by sender for grants that legitimately need to survive long offline gaps.

**Verdict: SOFT-BREAK for DeviceLink + RemotePermission; need U43 (offline-replay carve-out via `received_at_epoch` + opt-in `offline_acceptance_grace_seconds`).**

### Scenario E — Alice forks her Atrium; pre-fork Drop bundles + audience mismatch

**Setup.** Alice's Atrium A is membered by {Bob, Carol, Dave}. At T₁, Alice does an Atrium fork: post-fork-A retains {Bob, Carol}; post-fork-B retains {Dave}. Pre-fork Drop bundles had audience_did = {Bob, Carol, Dave}.

**What §6.2 + amendments specify.**
- Per ratified forkability semantic (`2026-05-27`): "member-leaves-keeps-past-content". Dave, even though no longer in post-fork-A, retains his stanza decap-keys for pre-fork Drops; Dave can still Open them. L9 §4.1.
- U17 multi-stanza ensures Dave's stanza is present in pre-fork Drop bundles.
- U18 dual-CID ensures pre-fork Drop `plaintext_cid` references survive the fork.

**Where it breaks.**
1. **Forkability semantic preserves pre-fork READ; what about post-fork CRDT-merge with pre-fork content?** Dave (post-fork-B) edits a Node that he received pre-fork. Dave's edit happens in post-fork-B's CRDT space. Bob (post-fork-A) edits the same Node in post-fork-A's CRDT space.
2. **Should these two edits ever attempt to converge?** Per the forkability semantic: NO — they're in different Atriums now. But:
   - If the Node's CID is the same pre-fork (Dave and Bob both have CID-X), and post-fork they both produce divergent edits, what prevents accidental cross-fork sync if any peer is dual-membered? (Ben hasn't ratified that membership is mutually-exclusive; in the general case, peers can be in multiple Atriums.)
   - Per L9 §4.1 + the forkability ratification, this is "fork-membership scopes the sync graph" — sync only happens within a fork. So Dave's edit lives in fork-B's CRDT timeline; Bob's in fork-A's. They never see each other. Good.
3. **Risk.** If membership-scoping is enforced by Atrium-layer convention rather than cryptographic enforcement, a buggy peer could leak post-fork content across forks. The defense is: each Atrium has a distinct `atrium_did` + sync-message-AAD-binds `atrium_did`; cross-Atrium sync messages structurally reject at HPKE-decap. This is an existing Atrium discipline — not a §6.2 envelope concern.
4. **Pre-fork Drop bundle CRDT-merge inside fork-A.** Bob's CRDT in fork-A merges pre-fork-Drop content normally. The U17 cross-stanza substitution defense binds `sorted recipient-DID-list` into per-stanza AAD; that recipient-list is the **pre-fork** {Bob, Carol, Dave}. Bob can still verify integrity. ✓
5. **Post-fork new Drops.** Alice in fork-A issues new Drops to {Bob, Carol} only; Dave's stanza is absent; AAD binds new recipient-list. The pre-fork→post-fork transition is wire-visible via the changed recipient-list; this is correct.

**Verdict: WORKS given the forkability semantic + Atrium-layer enforcement of fork-scoped sync. No new amendment needed.** But document explicitly: "U17 recipient-DID-list AAD-binding ALSO doubles as fork-epoch-marker; pre-fork vs post-fork Drops are distinguishable on the wire." Add as observation to U17 doc impact section.

### Scenario F — Concurrent ExecuteWorkflow (U21) on same input_node_cids

**Setup.** Alice grants two compute peers C1 + C2 the same `ExecuteWorkflow { workflow_cid: W, input_node_cids: [X, Y], max_decrypt_count: 1, result_recipient_pubkey: alice_pk, executor_did: ... }` per U21. C1 + C2 both decrypt X + Y + execute W + return results.

**What §6.2 + amendments specify.**
- U21 reserves the variant shape but defers full impl to post-v1-beta.
- L9 §6 discusses but doesn't pin concurrency semantics.

**Where it breaks (FUTURE-LOAD-BEARING; not v1-beta-day-one but the wire format freezes now).**
1. **Two compute peers race to execute.** Both Open X + Y, both run W, both produce a result Node-R-C1 vs Node-R-C2 (different result-CIDs if W is non-deterministic; same CID if deterministic).
2. **Result delivery.** Each compute peer encrypts result under alice_pk and Drops. Alice receives two result-Drops.
3. **CRDT merge.** Alice's CRDT sees two NodeUpdates for "result of W on (X, Y)". If W is deterministic, both have the same CID → trivial dedup. If non-deterministic, they have different CIDs → either Alice's CRDT picks one via HLC-LWW (silently discards the other compute peer's work) or carries both as alternatives.
4. **max_decrypt_count enforcement.** The amendment says C1 + C2 are each granted max_decrypt_count=1. Each consumes their 1; both successfully decrypt. The grant doesn't prevent **redundant** computation; it only caps per-peer decrypts.
5. **The actually-interesting CRDT property:** Alice's UI should display which compute peer's result she's using. The U21 BindingContext doesn't carry result-provenance back; the result-Drop wraps a generic plaintext. Recommend: U21 result-Drop's BindingContext::DropToRecipient extends with `result_of_execution: { executor_did, input_node_cids, workflow_cid }` so Alice's CRDT can dedup-merge by computation-identity rather than by result-content-CID.
6. **Conflicting-results dispute.** If C1 + C2 disagree (compute-peer-malice or compute-peer-bug), Alice's CRDT-layer cannot tell. The application layer needs an N-of-M agreement protocol on top. Out of §6.2 scope.

**Verdict: NEEDS REFINEMENT of U21 BindingContext to carry execution-provenance for CRDT-dedup.** Add as extension to U21 (in U21 doc impact section) — not a new amendment.

### Scenario G — K_principal rotation while a Drop bundle is in-flight

**Setup.** Sender encrypts Drop under K_principal-gen-N (per U20 binding `k_principal_generation: u32` into AAD). Sender's K_principal rotates to gen-N+1 between encryption and delivery. Sender's view of "current generation" is now N+1.

**What §6.2 + amendments specify.**
- U20 specifies `k_principal_generation: u32` in Layer-B AEAD AAD + Vault BindingContext.
- The KPrincipalRotation Atrium-replicated Node propagates the new generation to all of Alice's devices.
- **But:** Drop bundles to recipients (Layer-C/D, encrypted to recipient HPKE pubkey, NOT K(N)) are independent of K_principal generation. K_principal is sender-side at-rest keying; Drop bundles encrypt to recipient's pubkey directly.

**Where it breaks.**
1. **For a Drop containing K(N)-encrypted Nodes:** the Drop wraps K(N) for the recipient; K(N) is derived from sender's K_principal-gen-N. The wrapped K(N) decrypts independent of sender's current K_principal-gen. Recipient gets K(N), decrypts Layer-B node. ✓ (works correctly).
2. **For sender's own at-rest Layer-B reads after K_principal rotation:** sender's engine must derive K(N) under the OLD K_principal-gen-N for that Node, not the new gen-N+1. U20 covers this: sender holds `HashMap<u32, [u8;32]>` of historical K_principal generations. ✓
3. **What if K_principal rotation happens because of a security event** (compromised device; sender wants to deny the OLD K_principal access to NEW Nodes)? Sender's engine must re-encrypt all owned Nodes under new K_principal-gen-N+1 + zeroize old K_principal-gen-N. **This is the K(N) re-keying problem of Scenario A at the K_principal axis.** U20 doesn't specify the re-encryption discipline; it specifies the generation tracking.
4. **Recipient-side.** Recipient has copies of K(N) (received via prior Drops) for some Nodes. After sender's rotation + re-encryption, recipient's K(N) is for the OLD encrypted blob. Recipient needs to receive a NEW Drop containing the new K(N)-under-gen-N+1 for any Node sender wants recipient to continue accessing. This is a sender-side discipline: on K_principal-security-event-rotation, re-Drop all currently-shared Nodes to all current recipients. Cost: O(Nodes × Recipients) Drops. Heavy but unavoidable.

**Verdict: U20 is necessary but not sufficient.** Need U41-extension or a new sub-amendment specifying the **re-encryption discipline** on K_principal rotation: security-event-rotation requires re-encryption of all in-scope Nodes under new K_principal-gen; benign rotation (periodic) does not require re-encryption of past Nodes (old K(N) keys remain valid for past blobs; new Nodes encrypt under new gen).

---

## §3 Amendment-CRDT composition gap analysis (Task 2)

Per-amendment scan of the 28-amendment registry for CRDT-conflict handling:

| Amendment | CRDT-relevant? | Gap? | Notes |
|---|---|---|---|
| U1 codepoint in AAD | N | — | Wire framing; CRDT-orthogonal. |
| U2 strict-decode | N | — | Decoder discipline. |
| U3 canonical-TLV | N | — | Encoding injectivity. |
| U4 sender-DID in AAD | Y | **PARTIAL** | Sender-DID identifies device-of-origin for HLC peer-id attribution. If two devices have same DID (multi-process per CLAUDE.md #17), AAD-binding can't distinguish; HLC carries device-id. No conflict resolution. |
| U5 replay-window | Y | **GAP (Scenario D)** | Excludes Drops + Vault; does NOT exclude offline-receipt. **NEEDS U43.** |
| U6 Bernstein-Persichetti CT | N | — | Crypto-impl. |
| U7 BE codepoint | N | — | Wire framing. |
| U8 codepoint registry | N | — | Registry discipline. |
| U9 / U10 non_exhaustive | N | — | Permanence. |
| U11 escape codepoint | N | — | Permanence. |
| U12 nonce-length variant | N | — | Wire framing. |
| U13 FS-gap codepoint reserve | N | — | Reservation; no semantic. |
| U14 aad_version | N | — | Canonicalization version. |
| U15 Did multikey | N | — | DID encoding. |
| U16 CodepointLifecycle | N | — | Lifecycle states. |
| U17 multi-recipient stanza | Y | **GAP (Scenario B + Scenario E)** | Cross-stanza substitution defense binds recipient-DID-list; this list also serves as fork-epoch-marker. Doc-only extension needed. |
| U18 dual-CID | Y | **PARTIAL GAP (Scenario A + C)** | `plaintext_cid` semantics under CRDT-merge are undefined. Is it pre-merge or post-merge? **NEEDS U41.** |
| U19 recipient_key_generation | Y | OK | Recipient-key rotation generation tracking is well-defined. |
| U20 K_principal_generation | Y | **GAP (Scenario G + open: rotation-log concurrent-rotation)** | KPrincipalRotation Atrium-Node is itself CRDT-mutated; concurrent rotations from two devices produce divergent counters. **NEEDS U42.** Plus K_principal-rotation triggering re-encryption needs U41 discipline. |
| U21 ExecuteWorkflow | Y | **GAP (Scenario F)** | Result-provenance binding for CRDT-dedup of concurrent executions. **NEEDS U21 extension.** |
| U22 Sealed-Sender slot | N | — | Reservation. |
| U23 per-relay-unlinkability | N | — | Transport blinding. |
| U24 size-class padding | N | — | Privacy padding. |
| U25 per-recipient-unlinkable | N | — | Invariant. |
| U26 cover-traffic deferred | N | — | Deferred. |
| U27 DID-rotation deferred | N | — | Deferred. |
| U28 coarse epoch buckets | Y | **PARTIAL** | 1-hour bucket granularity matches HLC physical-clock granularity well; ensures replay-window math is hour-precise. Compatible with HLC. No new gap. |
| U29 cross-ecosystem mapping | N | — | Mapping table. |
| U30 DAG-CBOR | N | — | Outer framing. |
| U31 libcrux | N | — | Crate choice. |
| U32 XChaCha20 | N | — | AEAD choice. |
| U33 NAPI canonical_binding | N | — | Cross-lang. |
| U34 NAPI opaque-handle | N | — | NAPI pattern. |
| U35 wasm_js cfg | N | — | Build config. |
| U36 cancel-safety | N | — | Async discipline. |
| U37-U40 deliverables | N | — | Audit deliverables. |

**Summary.** 4 amendments have substantive CRDT-encryption composition gaps (U5, U17, U18, U20) + 1 has a refinement-need (U21). Three new amendments + one extension close them.

### MF6 — Pattern-induction observation (consolidator-MF style)

**Three of the four gaps share root cause: "K(N) and K_principal-generation are functions of mutable graph state (Node-CID, rotation-log) without explicit CRDT-resolution rules."** The encryption-substrate's keying functions assume the inputs are themselves resolved; the CRDT-substrate's job is precisely to resolve concurrent inputs. The seam is at the keying-function-input layer, not at the envelope-format layer. Fixing it well means: (a) make K(N) a function of `merged_plaintext_cid` post-CRDT-merge with a documented re-encryption discipline; (b) make K_principal-generation a CRDT-vector (per-device, last-rotated-by-device-DID) not a scalar counter; (c) bind `received_at_epoch` for time-bound credentials so offline-receipt is a first-class concept. The fixes generalize to "any future encryption substrate addition whose key derivation reads mutable graph state must specify the CRDT-conflict-resolution rule for that read."

This is a pattern-class observation worth codifying as **Inv-19**:

> **Inv-19 (proposed) — Encryption-substrate keying-function CRDT-input discipline.** Every key-derivation function in Benten's crypto substrate that reads mutable graph state (Node-CID, K_principal-generation, recipient-key-generation, rotation-log) MUST cite the CRDT-conflict-resolution rule for that input. Re-keying-on-CRDT-merge discipline MUST be specified at the same site as the KDF. Static keying-functions (codepoint, BE encoding, TLV structure) are exempt.

---

## §4 Recommended additional amendments (Task 3)

### U41 — CRDT-merge re-encryption discipline + AAD-bound predecessor chain

**Origin.** This lens (L11/U41), driven by Scenarios A + B + G.

**Statement.**
1. Layer-B per-Node AEAD MUST add an AAD-bound field `predecessor_plaintext_cid: Option<Cid>` (None for genesis Nodes; Some for any Node minted via CRDT-merge or version-chain-advance). The merged-Node's encrypted blob binds the immediate-predecessor's plaintext-CID so the version-chain is wire-attestable.
2. After CRDT-merge produces a new Version Node with plaintext-CID X3, the merging engine MUST: (a) derive K(X3) = KDF(K_principal-gen-current, X3); (b) encrypt the merged plaintext under K(X3) with AAD-binding `predecessor_plaintext_cid = Some(X3_pred_set)` (encoded as canonical-sorted CID list); (c) publish the new encrypted blob to Atrium; (d) MAY mark the predecessor encrypted blobs as GC-eligible after a grace period (recommend 30 days; cite history-replay use cases).
3. K_principal rotation triggered by security-event MUST be followed by re-encryption sweep: all owned Nodes under K_principal-gen-N+1 + re-Drop to current recipients. Benign-rotation (periodic) does NOT require re-encryption sweep (legacy K(N) remain valid for legacy blobs).
4. `plaintext_cid` field in U18 dual-CID MUST be defined as **post-merge plaintext-CID for any envelope carrying a CRDT-merged Node**, and pre-merge plaintext-CID for any envelope carrying a non-merged single-writer Node. Discriminator carried via the new `predecessor_plaintext_cid` field — None ⇒ non-merged; Some ⇒ merged.

**Severity.** LOAD-BEARING (wire-format-affecting: adds AAD field).
**Wire-affecting:** Y. **Impl-only:** N. **Codepoint-reserve:** N.
**Disposition.** v1-beta-LOAD-BEARING.
**Depends-on.** U1, U3, U17, U18, U20.
**Compromise/Invariant.** Inv-19 (proposed). Doc impact: `docs/ARCHITECTURE.md` CRDT-encryption-section, `docs/KEY-LIFECYCLE.md` re-encryption-on-rotation, `docs/INTERNALS.md` for `benten-sync` crate.
**Wave-day estimate.** ~3-4 days (AAD field addition + re-encryption-on-merge engine wiring + golden vectors).
**Confidence.** HIGH on the gap; MED-HIGH on the specific shape (the GC-grace-period value is judgment; 30 days is conservative).

### U42 — K_principal-rotation log CRDT conflict-resolution rule + vector-generation

**Origin.** This lens (L11/U42), driven by Scenario G + C4's named gap ("Atrium-CRDT-conflict-resolution at rotation-log layer").

**Statement.**
1. `k_principal_generation: u32` (U20) is INSUFFICIENT under concurrent multi-device rotation. Replace scalar `u32` with vector `{device_did → u32}` per-device rotation counter; the engine's effective K_principal at any HLC moment is determined by the device whose counter is highest at that HLC.
2. KPrincipalRotation Atrium-Node CRDT-merge rule: **per-device counter is monotonically non-decreasing** (multi-writer LWW per device-DID partition). When merging two divergent rotation-Nodes, take per-device max.
3. The effective-K_principal-at-HLC-T determination: at HLC-T, the K_principal generation in scope = the maximum across all device-DID partitions of (counter values rotated at HLC ≤ T). Provides deterministic Lamport-style resolution.
4. Layer-B AEAD AAD MUST bind the full `{device_did → counter}` vector AT-ENCRYPT-TIME (not just the scalar — there is no scalar). Recipient with full vector + Atrium-replicated KPrincipalRotation log derives correct K(N) deterministically.
5. Wire impact: U20's `k_principal_generation: u32` becomes `k_principal_generation_vector: BTreeMap<Did, u32>` (canonical-sorted by Did). Adds bytes proportional to device count; bounded by Benten's typical 2-5 devices/principal. ~30-80 bytes per envelope.

**Severity.** LOAD-BEARING (wire-format-affecting; replaces U20's scalar with vector).
**Wire-affecting:** Y. **Impl-only:** N. **Codepoint-reserve:** N.
**Disposition.** v1-beta-LOAD-BEARING.
**Depends-on.** U20 (extends/supersedes the scalar form).
**Compromise/Invariant.** Inv-19 (proposed). Doc impact: `docs/KEY-LIFECYCLE.md` rotation-log CRDT rule, `docs/INTERNALS.md`.
**Wave-day estimate.** ~2-3 days (scalar→vector field migration + CRDT-merge rule in `benten-sync` + golden vectors).
**Confidence.** HIGH on the gap; MED on the vector-vs-scalar decision (a single-device-rotates-at-a-time discipline would let scalar work but is fragile; vector is the structurally clean choice and matches Signal/MLS group-state-rotation tradition).

**Alternative.** Keep U20's scalar; mandate "K_principal rotation is strictly single-writer (rotation MUST be initiated from designated primary device)". Simpler wire; brittle UX (primary-device-offline ⇒ can't rotate). Consolidator-style judgment: the vector cost is small; UX brittleness is high; vector wins.

### U43 — Offline-replay carve-out for time-bound credentials

**Origin.** This lens (L11/U43), driven by Scenario D.

**Statement.**
1. Replay-window enforcement per U5 (`now() <= valid_until`) MUST be re-anchored to `received_at_epoch <= valid_until + offline_acceptance_grace_seconds` for DeviceLink + RemotePermission + ExecuteWorkflow envelopes.
2. `received_at_epoch: u64` is stamped LOCALLY by recipient on first decode-success + persisted in recipient's local-only state (NOT wire). Recipient maintains per-envelope-CID receipt-log; replay-after-stamp is rejected (replay defense survives).
3. Sender MAY opt-in to long offline-acceptance via `offline_acceptance_grace_seconds: u32` AAD-bound field on the BindingContext variant. Default = 0 (strict per existing U5 + Compromise #44 / BSI-strict). Common values: 0 (strict), 86400 (1 day), 2592000 (30 days), 31536000 (1 year). Maximum capped at 5 years (defense-against-permanent-bypass).
4. Sender's UI surface: when issuing a long-offline-grace grant, surface to user: "this grant remains acceptable to recipient even if delivered up to X after expiry (use case: recipient device may be long-offline)". User opt-in per-grant.
5. Receipt-log GC: receipt-log entries can be GC'd once `now() > valid_until + offline_acceptance_grace_seconds` (no future replay possible).

**Severity.** LOAD-BEARING (wire-format-affecting: adds optional AAD field; semantics-affecting for U5).
**Wire-affecting:** Y. **Impl-only:** N. **Codepoint-reserve:** N.
**Disposition.** v1-beta-LOAD-BEARING.
**Depends-on.** U5 (refines).
**Compromise/Invariant.** Inv-19 (proposed) — explicit offline-receipt is a CRDT-input. Doc impact: `docs/SECURITY-POSTURE.md` replay-window-semantics, `docs/INTERNALS.md` recipient-state for receipt-log.
**Wave-day estimate.** ~1-2 days (optional AAD field + recipient-state log + UI surface).
**Confidence.** HIGH on the gap; HIGH on the shape (matches UCAN + Macaroons + Signal precedent).

### U44 — Drop-bundle Node-update granularity + predecessor-CID chain (composes with U41)

**Origin.** This lens (L11/U44), driven by Scenario C + Scenario A composition.

**Statement.**
1. Drop bundles carrying `Vec<NodeUpdate>` MUST include per-update `expected_predecessor_plaintext_cid: Option<Cid>` (Some = recipient is expected to be at this predecessor; None = recipient may apply unconditionally).
2. Recipient's engine, on Open, checks per-update: if local CRDT has the predecessor, apply via CRDT-merge; if local CRDT has a DIFFERENT predecessor (CRDT-divergence), apply via U41 re-encryption discipline (recipient re-encrypts merged result + optionally re-Drops back to sender for round-trip).
3. The U17 cross-stanza substitution defense for HpkeMultiBase MUST be parameterized to distinguish "this stanza's recipient-DID-list differs from another stanza's recipient-DID-list" (substitution-attack signal) from "this stanza's `expected_predecessor_plaintext_cid` differs from recipient's local-state" (CRDT-divergence signal). Today U17 conflates both into a single AAD-mismatch failure.
4. Recipient surfaces CRDT-divergence as a non-error event (engine merges) vs substitution-attack as a hard-error (envelope rejected).

**Severity.** RECOMMENDED (wire-format-affecting: optional per-update AAD field; but error-surface discrimination is impl-only).
**Wire-affecting:** Y. **Impl-only:** partial. **Codepoint-reserve:** N.
**Disposition.** v1-beta-LOAD-BEARING (the wire field + AAD-binding); impl of error-discrimination is engine-side.
**Depends-on.** U17, U41.
**Compromise/Invariant.** Inv-19. Doc impact: `crates/benten-drop/INTERNALS.md` NodeUpdate shape + `docs/INTERNALS.md`.
**Wave-day estimate.** ~2-3 days (field addition + error-discrimination logic + golden vectors).
**Confidence.** MED-HIGH on the gap; MED on the shape (depends on Drop-bundle granularity decisions still pinning at R0).

### U21 extension — ExecuteWorkflow result-provenance binding

**Statement.** Extend U21's BindingContext variant: result-Drop carries `result_of_execution: { workflow_cid: Cid, input_node_cids: Vec<Cid>, executor_did: Did, executed_at_hlc: BentenHlc }` in BindingContext::DropToRecipient (or a new variant DropToRecipient::ExecutionResult). Alice's CRDT-merge dedups concurrent compute-peer results by `(workflow_cid, sorted_input_cids)` key; competing executor results presented to application layer for arbitration.

**Severity.** Extension to U21. Wire-affecting: Y. Disposition: v1-beta-LOAD-BEARING (composes with U21 reservation).
**Wave-day estimate.** ~0.5-1 day (incremental to U21's existing reservation work).

### Compromise #45 — Collaborative-edit-via-re-drop (accepted v1-beta trade-off)

**Statement.** True concurrent multi-writer encrypted CRDT collaborative edit at Node-granularity is **NOT supported at v1-beta** without explicit Drop-bundle ping-pong per edit. CGKA-based group-K(N) for true concurrent multi-writer is deferred to post-v1-beta per Compromise #42 (FS-gap). Application-layer use-cases requiring real-time concurrent multi-writer (e.g. Google-Docs-style co-editing) are not in v1-beta scope.

**Origin.** This lens, Scenario B.
**Wire impact.** N. **Doc impact.** `docs/SECURITY-POSTURE.md` + `docs/THREAT-MODEL.md`.

### Inv-19 — Encryption-substrate keying-function CRDT-input discipline

**Statement (proposed).** Every key-derivation function in Benten's crypto substrate that reads mutable graph state (Node-CID, K_principal-generation, recipient-key-generation, rotation-log) MUST cite the CRDT-conflict-resolution rule for that input. Re-keying-on-CRDT-merge discipline MUST be specified at the same site as the KDF. Static keying-functions (codepoint, BE encoding, TLV structure) are exempt.

**Composes-with.** Inv-16, Inv-17, Inv-18.
**Enforcement plan.** Cite-drift-detector scanner: every `KDF` / `derive_key` / `kdf` invocation in `benten-crypto-suite` + `benten-engine` + `benten-drop` that reads a Node-CID, generation-counter, or rotation-log MUST be accompanied by a `// CRDT-INPUT: <resolution-rule-doc-ref>` comment OR static-attribute. Scanner flags missing.

### Summary table

| # | Title | Severity | Wave-days | Confidence on gap |
|---|---|---|---|---|
| U41 | CRDT-merge re-encryption + predecessor-CID AAD | LOAD-BEARING | 3-4 | HIGH |
| U42 | K_principal-rotation vector + log CRDT-merge rule | LOAD-BEARING | 2-3 | HIGH |
| U43 | Offline-replay carve-out (received_at_epoch) | LOAD-BEARING | 1-2 | HIGH |
| U44 | Drop-bundle predecessor-CID chain + error-discrimination | RECOMMENDED | 2-3 | MED-HIGH |
| U21-ext | ExecuteWorkflow result-provenance | LOAD-BEARING (ext) | 0.5-1 | MED-HIGH |
| #45 | Collaborative-edit-via-re-drop accepted trade-off | Compromise mint | 0.5 (doc) | HIGH |
| Inv-19 | Crypto-keying CRDT-input discipline | Invariant mint | 0.5 (doc) + scanner | HIGH on direction |
| **Total** | | | **~10-14** | |

Fits inside the consolidator's ~13-15 week v1-beta envelope per §6 raising high-end to ~14-17 weeks.

---

## §5 Willow Confidential Sync learnings (Task 4)

Per WebFetch of `willowprotocol.org/specs/confidential-sync/`: Willow Confidential Sync explicitly delegates encryption-at-the-channel out of scope ("The handshake and encryption of the communication channel are out of scope of Confidential Sync"). What Willow DOES contribute is:

1. **Private Area Intersection.** Two peers determine which namespaces + Areas they share interest in WITHOUT leaking non-shared namespaces. Mechanism: each peer commits to a salted hash of their namespace-list; intersection computed via cryptographic set-intersection. **Benten-relevance.** Benten's Atrium-membership-overlap discovery for fork composition is analogous. Adopt the PSI primitive concept at post-v1 Atrium-discovery layer (NOT v1-beta).
2. **3D Range-Based Set Reconciliation.** Efficient diff of entry sets across peers via recursively-partitioned fingerprints. **Benten-relevance.** iroh-docs already uses range-based set reconciliation (per WebSearch); Benten's Atrium-CRDT sync could adopt similar techniques. NOT a §6.2 envelope concern.
3. **Stores form a state-based CRDT under the join operation.** Willow's data model is a state-based CRDT; this is the structural foundation that lets eventually-consistent sync work over arbitrary message orderings. **Benten-relevance.** Benten's `benten-sync/src/crdt.rs` uses Loro (op-based-with-snapshot) which is a similar shape. Already aligned.
4. **Confidential Sync does NOT specify how to merge encrypted entries.** Willow assumes entries are immutable (the data-model commitment) — so "merge" is set-union; no two distinct entries ever need to be merged into a single entry. **The Willow data-model commitment to immutability is itself the answer to the encrypted-CRDT-merge question.** Benten chose mutable Nodes with version-chains + CRDT merge; this introduces the K(N) re-keying problem Willow doesn't have.

**Load-bearing learning for Benten v1-beta.**

> **Willow learns by NOT having the problem.** The Willow data-model commits to entry-immutability so encrypted-entry-CRDT-merge is set-union; no two encrypted entries are ever combined into one. Benten's mutable-Node + version-chain model creates the K(N) re-keying problem (Scenario A) that Willow doesn't have. **Two paths.** (A) Stay with mutable-Nodes + add U41 + U42 + Inv-19 to make the re-keying discipline explicit. (B) Constrain Benten's encrypted-Node-Layer-B to immutable-blob shape (every "edit" mints a new Node-CID; CRDT-merge produces a new immutable Node; old K(N) blobs remain valid forever; GC by reference-counting). Path B is structurally simpler + matches Willow + iroh-docs + IPFS-style content-addressing pure form; Path A preserves Benten's existing version-chain pattern.

**CONSOLIDATOR-STYLE RECOMMENDATION.** Adopt Path A (U41 + U42 + Inv-19) for v1-beta because version-chains are already a 5-week-old structural commitment in `arch-r1-4` D-C HYBRID. Document Path B as a future-architecture-revisit-trigger candidate post-v1-GM if K(N) re-keying overhead proves operationally heavy. Add to V1-FROZEN-INTERFACE-DEFERRED.md.

5. **Sources for Willow.** Primary: [Willow Confidential Sync spec](https://willowprotocol.org/specs/confidential-sync/index.html). Secondary: [jzhao.xyz Willow notes](https://jzhao.xyz/thoughts/Willow-Protocol).

---

## §6 Encrypted-CRDT precedent table (Task 5)

| System | CRDT model | Encryption model | Key rotation | Merge-of-encrypted? | Benten alignment |
|---|---|---|---|---|---|
| **Yjs (vanilla)** | Op-based RGA + custom for lists/maps | None at protocol; transport e2ee external | N/A | N/A: merges plaintext | — |
| **Yjs + Serenity Notes** | Yjs op-based | App-level symmetric key per document | Manual rotation; rare in practice | Plaintext-CRDT under one key; merge happens after Open | Path-B-shape: single key per doc, no per-Node re-keying |
| **Automerge** | Op-based JSON-CRDT | None at protocol; e2ee external | N/A | N/A: merges plaintext | — |
| **CRDX (Herb Caudill)** | Custom op-based | Encrypted at rest + in transit | Lockbox key-rotation pattern: every key change emits a "lockbox" with new key sealed to each authorized member | Plaintext-CRDT after Open; lockboxes coordinate rotation | Closest precedent for Benten's K_principal-rotation; the "lockbox" pattern is analogous to U20's KPrincipalRotation Node |
| **secsync (Nik Graf)** | Yjs op-based | Snapshot+updates model; symmetric ratchet key per snapshot | Symmetric KDF-ratchet inspired by Signal Private Group; rotation = new snapshot + relay drops old | Plaintext-CRDT inside snapshot; updates encrypted under snapshot key | Closest production precedent for snapshot-based encrypted-CRDT; relay-drops-old-snapshot is Path-B-shape |
| **Jazz (CoJSON)** | Custom Co-Values; CRDT-style | Group-based access control via encryption-signature; group-key rotated on member removal | Member removal triggers key rotation + clients refetch latest snapshot | Plaintext-CRDT after Open; group-key change = recompute downstream | Closest design-language match for Benten's group-K(N) future via CGKA |
| **iroh-docs** | Range-based set reconciliation; entries are immutable | Channel-level QUIC e2ee only; no app-level encryption | N/A app-level; channel key per QUIC session | N/A: entries immutable | Path-B exemplar: immutability sidesteps re-keying |
| **Willow** | State-based CRDT under join; entries immutable | Out of scope at Confidential Sync layer | N/A at protocol | N/A: entries immutable; join is set-union | Path-B exemplar |
| **Signal Sealed Sender + groups** | N/A (not a CRDT) | E2EE per message; CGKA-managed group key | CGKA epoch ratcheting | N/A | Precedent for Benten's deferred CGKA |
| **Benten (current F-full)** | Per-property HLC-LWW Loro + version-chains (D-C HYBRID) | Per-Node AEAD K(N) = KDF(K_principal, N.cid); HPKE for sharing | K_principal-generation U20 (scalar); recipient-key-generation U19 | **AMBIGUOUS** — engine re-encrypts merged Node; not specified | Closest to CRDX (per-principal-key + rotation log); diverges on per-Node-CID keying which CRDX does not have |

**Pattern.** Successful encrypted-CRDT systems fall into TWO clean shapes:
- **Path B: Immutable-entries.** Willow, iroh-docs, secsync-snapshot-style. "Encrypted CRDT" is "encrypted blobs + plaintext-CRDT after Open + immutability commitment". K-rotation triggers fresh snapshot, NOT re-encryption-in-place. **Cleanest; widely adopted; matches Web2 e2e tradition.**
- **Path A: Mutable-entries with lockbox/group-key.** CRDX, Jazz, Signal-groups. Per-entry encryption key but a small number of keys (one per access-group); rotation = new lockbox to current members. Re-encryption-on-rotation is bounded (entries-in-scope × members).

**Benten's current design is a UNIQUE THIRD PATH.** Per-Node-CID-derived K(N) where N.cid is mutable graph-state. **No precedent surveyed.** The IPFS / content-addressing crowd uses BLAKE3-CID keying for chunk-encryption (e.g. age-of-empires-style file encryption) but those CIDs are over IMMUTABLE chunk content. Benten's K(N) is over Node-CID which evolves under CRDT-merge — this is the novel feature.

**Recommendation.** Benten's third path is not WRONG; it's UNDER-SPECIFIED. The U41 + U42 + Inv-19 amendments codify the specification + provide an audit-defensible story. Long-term, post-v1-GM, consider Path-B migration if operational cost proves heavy. Either way, the third-path commitment should be disclosed in `docs/SECURITY-POSTURE.md` as a design-novelty (not honestly-disclosed-as-Compromise per se, but as "design-rationale-novel").

**Sources.**
- [Yjs main repo](https://github.com/yjs/yjs) (CRDT semantics).
- [Automerge CRDT concepts](https://posit-dev.github.io/automerge-r/articles/crdt-concepts.html).
- [CRDX repository](https://github.com/herbcaudill/crdx) (lockbox pattern).
- [secsync repository](https://github.com/nikgraf/secsync) (snapshot-based encrypted-CRDT).
- [Jazz / CoJSON](https://gitnation.com/contents/jazz-build-real-time-local-first-react-apps-with-sync-and-secure-collaborative-data) (group-based CRDT encryption).
- [iroh-docs](https://github.com/n0-computer/iroh-docs) (range-based set reconciliation; immutable entries).
- [Willow Confidential Sync](https://willowprotocol.org/specs/confidential-sync/index.html).

---

## §7 Layer-of-abstraction recommendation (Task 6)

Three options framed in brief:
- **(a) Envelope-format layer** (more amendments).
- **(b) Application layer** (engine handles CRDT; envelopes are opaque transport).
- **(c) Atrium-coordination layer** (Atrium maintains version-vectors; engine consults).

**Recommendation: (a)-with-(c)-support.**

### Why not pure-(b)

The "envelopes are opaque transport" stance fails empirically. The 9-eyes consolidated registry already commits envelopes to carry:
- Sender DID (U4) — application identity.
- Sealed-at + valid-until epochs (U5) — temporal semantics.
- Per-stanza recipient-DID-list AAD (U17) — group membership.
- `plaintext_cid` (U18) — graph identity.
- Recipient key generation (U19) — recipient state.
- K_principal generation (U20) — sender state.
- ExecuteWorkflow inputs + executor-DID (U21) — execution semantics.

The envelope is **already a semantic carrier**. Pretending it's opaque is incoherent; adding U41+U42+U43+U44 amendments brings the carrier in alignment with the CRDT semantics it has been implicitly making.

### Why not pure-(c)

Atrium-coordination layer alone fails because the engine needs to know — AT DECODE TIME — which K_principal-generation to derive K(N) from, and the answer is "it's bound in the AAD". If the engine had to consult Atrium-state to determine K(N), every Open would require a round-trip to Atrium (or a cached snapshot of Atrium-state) before crypto could begin. AAD-bound is faster + locally-verifiable.

### Why (a)-with-(c)-support

- **(a) Envelope-format amendments** (U41-U44) ensure the wire carries enough state for unambiguous local decode + CRDT-merge.
- **(c) Atrium-coordination layer** maintains the **convergence-discipline** for state that no single envelope can carry — specifically, the KPrincipalRotation log (U42) is itself an Atrium-replicated CRDT-state whose convergence rule MUST be specified at the coordination layer. Similarly the per-fork-membership-snapshot used by U17 for substitution defense.
- **Application layer** stays below the seam: application-layer CRDT operations (HLC-LWW, version-chain advance) happen on plaintext after Open + before Seal. Engine-side. No application-layer encryption awareness needed.

### Concrete layering

```
Application code (graph ops; subgraph spec; queries)
        ↓ engine API
Engine (HLC + CRDT merge + version-chain mint; PRE-ENCRYPT)
        ↓ Seal
Crypto-substrate (K(N) derivation; AEAD; HPKE; AAD binding) [F-full]
        ↓ envelope bytes
Atrium-coordination (KPrincipalRotation log; recipient_key_generation
                     advertising; per-fork-membership snapshot)
        ↓ sync messages
Transport (iroh-blobs; sendme; QUIC)
```

**Crisp seam:** application is CRDT-aware; engine is CRDT-and-crypto-aware (mints K-merged-cid + re-encrypts); crypto-substrate is encryption-only-aware (no CRDT logic, but AAD-binds the CRDT-state inputs the engine provides); Atrium-coordination is rotation-log-CRDT-aware (closes the rotation-counter-vector convergence gap U42 names).

---

## §8 Self-assessment + confidence (Task 7)

### What I did + how I worked

1. Tree-state pre-flight (clean at `2172cb6d`).
2. Branched + read 9-eyes consolidated registry in full (939 lines).
3. Read L9 atrium-integration review in relevant sections (~150 lines).
4. Read `crates/benten-sync/src/crdt.rs` head (per-property HLC-LWW Loro implementation).
5. Searched ARCHITECTURE.md + critique-c4-process-discipline for CRDT-coverage-gap mentions.
6. WebSearch / WebFetch for Willow Confidential Sync + Yjs/Automerge/Jazz/CRDX/secsync/iroh-docs encryption patterns.
7. Walked Tasks 1-7 in order; composed §1-§7 + §9.

### Confidence summary

| Section | Confidence | Rationale |
|---|---|---|
| §1 verdict + 4 gaps named | **MED-HIGH** | Gap-1 (K(N) re-keying) is HIGH confidence — structural to Layer-B keying. Gap-2 (rotation-log CRDT) is HIGH — U20 explicitly names Atrium-replication. Gap-3 (replay-window) is HIGH. Gap-4 (Drop-bundle predecessor) is MED. |
| §2 7 scenarios walked | **MED-HIGH** | Scenarios A + D + G are tightest; B + F have judgment-calls about Benten's specific use-case scope; C + E rest on Ben's ratified forkability semantic. |
| §3 amendment-by-amendment scan | **HIGH** | Mechanical scan of 28 amendments; CRDT-relevance determination is grounded in U#-text. |
| §4 4 new amendments + extension + Compromise + Invariant | **MED-HIGH** | Shape of U41+U42+U43 is HIGH; specific field values (GC-grace-period, max offline-grace, etc.) are MED — needs R0 §4 design pass + R3 implementer brief calibration. |
| §5 Willow learnings | **HIGH** on Path-A-vs-Path-B framing; **MED** on Benten-third-path novelty claim (I surveyed 8 systems; may have missed precedent for per-mutable-Node-CID encryption). |
| §6 precedent table | **MED-HIGH** | Table is comprehensive within surveyed-set; "no precedent" claim is bounded by my survey scope. |
| §7 (a)-with-(c) layering | **HIGH** | Direct consequence of the empirical fact that envelopes already carry semantic state per the 28-amendment registry. |

### What I could be wrong about

1. **The CRDT-merge re-encryption discipline (U41) may have an even simpler shape.** If Benten engine's typical operating model is "the merging device IS the device that holds K_principal" (rather than relay-side merge), then U41's "publish new encrypted blob back to Atrium" requirement is trivial. If relay-side merge is supported at v1-beta, U41 becomes structurally heavier. R0 §4 design should pin this.
2. **The K_principal-rotation-vector U42 may be overkill if Benten's UX commits to single-device-rotation-authority.** If Ben commits at R0 to "rotation requires primary-device", a scalar suffices + U42 collapses to "document the single-writer rule + reject concurrent-rotation". Wave-day cost halves.
3. **Compromise #45 (collaborative-edit-via-re-drop) may be unnecessary** if Benten's v1-beta use-case explicitly excludes Google-Docs-style real-time collab. If the v1-beta positioning is "personal-knowledge-graph + asynchronous sharing", then no compromise is needed — it's just out-of-scope by-construction. Recommend keep #45 as honest-disclosure regardless.
4. **Path-B Willow-alignment may be a better v1-beta call than Path-A** if Ben's structural preferences lean toward immutability. The 5-week-old D-C HYBRID commitment is reversible if Ben names it. Path-B reduces U41 to nothing + removes Inv-19. Worth a Ben call.
5. **U21 ExecuteWorkflow result-provenance extension** is judgment-bounded; the v1-beta-day-one implementation depth of U21 is MED per consolidator §2; if U21 stays purely reservation at v1-beta, my extension can defer to post-v1-beta with U21 itself.
6. **I did not survey CGKA-deferred dependencies for L11 amendments.** U41 + U42 may have additional shape when CGKA lands post-v1-beta; the AAD-binding fields I propose may not compose well with future CGKA-derived group-K(N). Recommend revisit at CGKA-design-pass.

### Lower-confidence areas (honest disclosure)

- I did NOT walk the full L1-L8 lens review files (only L9 + the consolidated registry + C4 critique). Some CRDT-relevant observations in L1-L8 may be unmentioned.
- I did NOT verify `crates/benten-sync/src/atrium*.rs` against my claims about Atrium-replicated rotation-log semantics; I propagated U20's text. If Atrium's actual replication discipline differs from naive CRDT-Node-add, U42 may need different shape.
- I did NOT cross-check against the RATIFIED-sharing-and-confidentiality-2026-05-21.md document (the brief cites it, but no such file exists in any git ref I searched; presumably memory/orchestration-internal). My claims rest on the registry's reflection of that doc's content.
- I did NOT examine whether iroh-blobs supports the "engine re-encrypts merged Node + publishes new blob + GCs old" cycle efficiently; if blob-publish has high overhead, U41's re-encryption discipline may need batching.
- The Willow precedent claim "Confidential Sync delegates encryption out of scope" is from a partial WebFetch; the full spec may contain encryption-relevant material I did not reach.

### What this lens does NOT cover

- CRDT correctness of `benten-sync/src/crdt.rs` independent of encryption — already pinned by existing property tests + out of scope here.
- Atrium peer-discovery + iroh-blobs replication — L9 covers (MED confidence per L9 §5.4).
- Formal-methods tractability of encrypted-CRDT-merge under adversarial model — C5 formal-methods lens scope.
- Performance / throughput of re-encryption sweeps on K_principal-rotation — needs benchmarking; defer to post-R0 perf-lens.
- UX surfaces for the offline-acceptance-grace-seconds opt-in dial — needs UX-affordance pass; consolidator MF5 named the UX-lens gap.

---

## §9 Citations

### Internal Benten references

- `phase-4-meta-core/option-f-plus-9-eyes-consolidated-registry @ fbdfeb16` — `.addl/phase-4-meta/option-f-plus-9-eyes-consolidated-registry.md` — 28 unified amendments + 13 Compromises + 3 invariants + 5 disagreements + cost estimate.
- `phase-4-meta-core/option-f-plus-lens-l9-atrium-integration @ 1670aa03` — `.addl/phase-4-meta/option-f-plus-lens-l9-atrium-integration.md` — A1-A5 amendments; §5.1 100-Drop-bundle catch-up scenario explicitly parked the CRDT-merge question this lens picks up.
- `phase-4-meta-core/option-f-plus-critique-c4-process-discipline @ 6ea9718a` — `.addl/phase-4-meta/option-f-plus-critique-c4-process-discipline.md` — §3.1 "No Atrium-CRDT-conflict-resolution lens" named the gap this lens fills.
- `origin/main` — `crates/benten-sync/src/crdt.rs` — Loro per-property HLC-LWW CRDT implementation; D-C HYBRID merge-to-Version-Node pattern; D-PHASE-3-4 + D-PHASE-3-22 RESOLVED.
- `origin/main` — `docs/ARCHITECTURE.md` lines 95-130 — `benten-sync` crate description ("CRDT for per-property LWW"); Phase-3 G16-B context.

### Cross-amendment cross-references

- U5 (replay-window) — extended by U43 (offline-replay carve-out).
- U17 (multi-recipient stanza) — composed with U44 (per-update predecessor chain + error-discrimination).
- U18 (dual-CID) — extended by U41 (post-merge plaintext_cid semantics).
- U20 (K_principal_generation scalar) — superseded by U42 (vector form).
- U21 (ExecuteWorkflow) — extended for result-provenance.
- Compromise #31 (forever-valid Drops) — composes cleanly with U43 (Drops + Vault remain excluded from time-bound per existing U5; U43 only affects DeviceLink + RemotePermission + ExecuteWorkflow).
- Compromise #42 (Layer-C FS-gap) — Compromise #45 cross-links (CGKA-derived group-K(N) closes both).

### External standards + production systems

- Willow Confidential Sync — [willowprotocol.org/specs/confidential-sync](https://willowprotocol.org/specs/confidential-sync/index.html); [jzhao.xyz/thoughts/Willow-Protocol](https://jzhao.xyz/thoughts/Willow-Protocol).
- Yjs (CRDT) — [github.com/yjs/yjs](https://github.com/yjs/yjs); [docs.yjs.dev](https://docs.yjs.dev/).
- Automerge (CRDT) — [posit-dev.github.io/automerge-r CRDT concepts](https://posit-dev.github.io/automerge-r/articles/crdt-concepts.html).
- CRDX (Herb Caudill; lockbox pattern) — [github.com/herbcaudill/crdx](https://github.com/herbcaudill/crdx).
- secsync (Nik Graf; snapshot+ratchet pattern) — [github.com/nikgraf/secsync](https://github.com/nikgraf/secsync); [NLnet SecSync project page](https://nlnet.nl/project/Naisho/).
- Jazz / CoJSON — [gitnation.com Jazz talk](https://gitnation.com/contents/jazz-build-real-time-local-first-react-apps-with-sync-and-secure-collaborative-data).
- iroh-docs — [github.com/n0-computer/iroh-docs](https://github.com/n0-computer/iroh-docs); [docs.rs/iroh-docs](https://docs.rs/iroh-docs/latest/iroh_docs/).
- Signal Sealed Sender + groups (CGKA) — Signal blog 2018 + 2024 V2; RFC 9420 MLS (referenced by consolidated registry §10.3).
- HLC (Hybrid Logical Clocks) — Kulkarni-Demirbas-Madappa-Avva-Leone 2014 "Logical Physical Clocks and Consistent Snapshots" — Benten's `BentenHlc` reflects this design.
- Loro — `loro` crate, used by `benten-sync` per per-property HLC-LWW pattern.

### Discipline references

- `feedback_handoff_top_banner_re_orient` — read CLAUDE.md + MEMORY.md banner discipline.
- `feedback_no_defer_HARD_RULE` — only 3 valid non-fix-now: OUT-OF-SCOPE / BELONGS-NAMED-NOW / DISAGREE-WITH-EXPLANATION. This lens's "needs new amendments" recommendations fall into BELONGS-NAMED-NOW (specific destination = registry §2 group extension + Inv-19 mint).
- `feedback_extra_reflection_pass_for_elegant_permanent_shape` — MF6 + Inv-19 proposal is the "one elegant structural shape closing N findings at once" pass.
- `feedback_review_finding_ground_truth_verify` — orchestrator should independently ground-truth-verify the K(N) re-keying gap by tracing `benten-sync/src/crdt.rs` D-C HYBRID merge path against the proposed F-full Layer-B `K(N) = KDF(K_principal, N.cid)` derivation.
- `feedback_pim_cross_language_rule_mirror` (§3.5g) — U41 + U42 AAD-binding additions MUST be mirrored in NAPI TS error catalog if any new ErrorCode variants land (e.g. `E_CRDT_MERGE_REENCRYPT_FAILED`, `E_RECEIPT_OFFLINE_GRACE_EXCEEDED`).

---

**End of L11 CRDT / eventual-consistency / conflict-resolution / offline-first lens review.**

**Summary handoff to orchestrator:** F-full §6.2-with-9-eyes-Amendments-U1-U40 composes acceptably but not seamlessly with Benten's per-property HLC-LWW Loro CRDT + D-C HYBRID version-chain merge layer. Four CRDT-encryption composition gaps identified, three with HIGH confidence: (Gap-1) K(N) = KDF(K_principal, N.cid) breaks under CRDT-merge that changes Node-CID — no amendment specifies re-encryption discipline; (Gap-2) U20's KPrincipalRotation Atrium-Node is itself CRDT-mutated and U20's scalar `k_principal_generation: u32` cannot represent concurrent multi-device rotation; (Gap-3) U5's replay-window has no offline-receipt carve-out and legitimately-offline DeviceLink + RemotePermission receivers structurally cannot honor expired-but-honest grants; (Gap-4) U17 cross-stanza substitution defense conflates substitution-attack with CRDT-divergence in Drop bundle Node-updates. Four amendments + one extension + one Compromise + one Invariant proposed (U41, U42, U43, U44, U21-ext, #45, Inv-19) totaling ~10-14 incremental wave-days. Path-A (mutable-Nodes with discipline) vs Path-B (Willow/iroh-docs immutable-entries) framing surfaced as a structural Ben call worth considering pre-R0 — Path-B simpler but requires reversing the 5-week-old D-C HYBRID commitment. Recommend Ben ratify U41-U44 + Inv-19 alongside the 9-eyes registry before F-full ADDL R0 plan-doc authoring proceeds.
