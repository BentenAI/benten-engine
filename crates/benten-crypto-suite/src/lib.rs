//! `benten-crypto-suite` — the ONE thin Benten-owned signature / hash /
//! cipher-suite agility integration crate.
//!
//! # RED-PHASE STUB (pim-12 §3.6e) — Phase-4-Meta-Core R3-A landing
//!
//! This crate is an **intentionally empty stub** at R3-A. Its real public
//! surface — the codepoint-dispatched signature/hash/cipher-suite seam
//! wrapping vetted upstream RustCrypto primitives — is the **G-CORE-2
//! (#1300) canary deliverable**, NOT this R3's job.
//!
//! The TF-2 test pins under `tests/` reference the *intended* G-CORE-2
//! public API (`benten_crypto_suite::sig::*`, `benten_crypto_suite::codepoint::*`,
//! `benten_crypto_suite::hash::*`, the size-touching envelope shapes). Those
//! `use` / symbol-resolution lines are the canonical RED-phase failure point:
//! they **compile-but-fail until G-CORE-2 lands the real crate**. Reviewers at
//! the G-CORE-2 closing wave MUST verify the `#[ignore]`d pins are
//! *un-ignored* (landing-status, not just spec-pin presence) per §3.6e.
//!
//! # The crypto-agility contract this crate will implement (G-CORE-2)
//!
//! Per CLAUDE.md #5 + `RATIFIED-crypto-agility-2026-05-18.md` +
//! `RATIFIED-pq-default-reframe-2026-05-19.md`:
//!
//! - The permanent commitment is the **multiformats framing** (CIDv1 /
//!   multihash / multicodec / `did:key` / UCAN-Varsig). Algorithms are
//!   v1 *default implementations selected within* that framing.
//! - **This crate is the ONLY crypto-primitive call site.** #1301 and the
//!   hash seam call its *typed* API; they NEVER instantiate a primitive.
//! - **Never fork / never reimplement primitives** — glue only
//!   (concat / hash / codepoint / envelope).
//! - **Never hardcode key/sig/ciphertext sizes.** Codepoint dispatch has a
//!   typed *unsupported-algorithm* arm — **never a silent fallback**.
//! - v1-beta signature default = **hybrid Ed25519⊕ML-DSA-65**, built
//!   **concatenated / committing / strip-resistant** (NF-4, §8 DECIDED
//!   option (a); `lamps-pq-composite-sigs-18`): both signatures travel
//!   together; **both must verify**; neither half can be stripped or
//!   substituted without the verify failing closed.
//! - Classical-only Ed25519 is a built **non-default downgrade** arm.
//!
//! G-CORE-2 fills `sig` / `hash` / `codepoint` (+ the #1301 cipher-suite
//! dispatch lands at G-CORE-3, calling this crate's typed API).
