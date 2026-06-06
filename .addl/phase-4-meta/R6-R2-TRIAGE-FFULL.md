# R6 Round-2 Triage — Phase-4-Meta-Core PHASE-CLOSE council (post-F-full, 2026-06-06)

(Distinct from the 2026-05-24 `R6-R2-TRIAGE.md` = the pre-F-full R6-R2 FP cycle. This is the post-F-full phase-close convergence, round 2; round 1 = `R6-R1-TRIAGE.md`.)

**Run:** `f-full-r6-council.js` round 2, Task `w3q83jska` / Run `wf_040ac861-436` (34 agents, 16 lenses [14 must-include + 2 bespoke: aad-injectivity-domain-sep + sealed-sender-metadata-leakage], ~90 min). Artifact = main `d0ccf606`.

**VERDICT: NOT-CONVERGED — 1 BLOCKER + 1 MISSED-LENS (root-linked) · 4 MAJOR · ~30 MINOR/OBS.** Byte-level crypto re-verified CLEAN (LAMPS byte-faithful, X-Wing label-appended, K(V) full-36B-CID, codepoint registry typed-reject, BE/length-injective/blinded AAD, random AEAD nonce, Inv-15..22 + Compromise #30..64 coherent). The blocker is a stated-guarantee falsity, not a wrong byte.

## ⛔ GAP-1 (BLOCKER) — SURFACE TO BEN (freeze fork)
**Layer-C online Sealed-Sender provides NO cryptographic sender-origin authentication, but SECURITY-PROOFS.md §4.1 affirmatively claims "inter-member non-forgeability."** Ground-truth @ d0ccf606: CEK = `BLAKE3(domain‖recipient_pk‖sender_did‖aad‖body)` — NO sender secret/signing key enters; recipient keypair derives from the PUBLIC recipient fingerprint, so ANY sealer can wrap to the recipient; `open_inner` does no origin check (only `InnerSenderDidForged` on a malformed length prefix). **Consequence:** any party (any group member for 0x6610/0x6520; any sealer for 0x6510) can mint a valid envelope attributing an ARBITRARY `sender_did`, and `open_*` returns it as "verified." §4.1 proof + `open_single` rustdoc + `f_lc_3` test all overclaim (f_lc_3 is SHAPE-not-SUBSTANCE — tampers ciphertext, never a second-sealer spoof). **Key context:** the OFFLINE `DropBundle` path (`envelope_sig.rs`) HAS real Ed25519 issuer auth — the gap is path-specific to the ONLINE Layer-C `EncryptedEnvelope`; groups are where spoofing bites (member impersonation).
- **B1 (council-rec, no wire change):** retract §4.1 non-forgeability → restate as sender-CONFIDENTIALITY-only; mint a named Compromise + THREAT-MODEL row "online Layer-C sender-DID is recipient-confidential but NOT origin-authenticated; trusted-sender rides DropBundle envelope-sig / upper-layer signature"; fix rustdoc/`InnerSenderDidForged`/`f_lc_3`. Optionally reserve a future authenticated-sealed-sender codepoint per crypto-agility.
- **B2 (make it true, wire change):** bind an inner sender signature (Varsig) → real non-forgeability. Changes the frozen Layer-C wire (additive codepoint per #5); per-message sig cost (~3373B LAMPS) + sender-signing-key mgmt.
- **MISSED-LENS** `sender-origin-authentication / sender-spoofing` → ADD round 3 (no panel lens examined spoofability; how §4.1 survived 18 lenses).

## MAJOR — FIX-NOW (round-2-fix wave)
- **F-01** boundary FORBIDDEN_DIRECT_DEPS omits live libcrux-ml-kem/sha3/sha2/secrecy + dead `ml-kem` entry.
- **F-03** F-full wave (15th crate + layer_d + layer_c) never got the §11 `#[non_exhaustive]` sweep (same class round-1 fixed for drop).
- **F-06** zero Layer-D + Layer-C-drop rows in V1-WIRE-FORMAT-INVENTORY.md (Ben P-III freeze sign-off deliverable) — cite f_ld_2/4/8 + f_02 golden pins.
- **F-07** cross-doc ratification-date drift: #30 + self-test "ratified Ben 2026-06-06" vs #39 + exemptions.toml header "pending ratification" (§3.5g).

## MINOR/OBS — FIX-NOW cluster (doc/comment/config)
F-08 (false "### Compromise #43") · F-11 (swap_matrix HKDF-SHA256→SHA3-256) · F-12 (primitives.rs "G-CORE-3 will add"→shipped) · F-13 (canonical-bytes LE/#[ignore]→BE/live) · F-15 (AAD body_cid no length-prefix width-check) · F-16 (cargo-vet "5+13=18"→13-of-18) · F-19 (AAD golden "131"→127B) · F-20 (member.rs decl-order vs key-sorted CBOR) · F-37 (sizes.rs "arms below" nonexistent) · GAP-2 (deterministic-CEK confirmation-oracle — disclose; random nonce keeps confidentiality) · F-02/F-09 (grep-fence/wasm-rung tripwire gaps).

## NAMED-CARRY (HARD-RULE clause-b)
F-04/F-05 (§11↔§16 non_exhaustive) · F-14 (privacy.rs input-independent → v1-GM) · F-17 (§16 ~8 rows vs 17 pub modules → Ben §16) · F-18 (DeviceLink 0x6310..0x631F absent from CRYPTO-CODEPOINTS) · F-21 (f_aad_2_nine_tuple→11-field) · F-22 (remote_permission "R5-FOLD-IN" stale) · F-23 (wasm-browser blocklist) · F-24/25/29/30/34 · F-26 (SHA 84280d31→d0ccf606 + Inv-21 ::crdt) · F-27 (f_kat_4 INBOUND live; OUTBOUND→v1-GM) · GAP-4 (Inv-15 registered-not-fully-enforced → G-CORE-PQ-WIRE-1 ledger).

## BEN-GATED FREEZE ITEMS (pre-tag)
GAP-1 (priority) · F-40 §16 §1.A.FROZEN inclusion sign-off · GAP-3/F-29 gossip §3.9-vs-§3.10 (freeze decision-sheet w/ K(V)).

## Sequence
SURFACE GAP-1 → Ben (B1/B2) → round-2-fix wave (GAP-1 per Ben + F-01/03/06/07 + FIX-NOW MINORs + NAMED-CARRY) → re-run R6 round 3 (ADD `sender-origin-authentication` lens; re-point artifact) → iterate to 0 BLK/MAJ → pre-tag (Ben §16 + gossip) → tag `phase-4-meta-core-close` (HOLD Ben).
