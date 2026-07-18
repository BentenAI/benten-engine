//! GAP-KDB Shape-B — the content-addressed `KeySetDocument` (design §1.2).
//!
//! A [`KeySetDocument`] is the CLOSED 5-field canonical DAG-CBOR map a
//! `did:benten` commits by CID: `{v, kem, sig, kem_cp, sig_cp}` (canonical
//! key order: length-first then bytewise — `v`(1) < `kem`(3) < `sig`(3) <
//! `kem_cp`(6) < `sig_cp`(6)). Its CID is
//! `self_describing_cid(BLAKE3-256(canonical_dagcbor(doc)))` — the DID **is**
//! the content-address of the key-set (pure Inv-15 / dispatch-conventions
//! §3.5s). See [`crate::did::Did::resolve_kem`] for the fail-closed KEM-key
//! recovery this commitment enables.
//!
//! # Freeze surface (v1-beta — Phase-4-Meta-Core)
//!
//! - Schema: exactly `{v, kem, sig, kem_cp, sig_cp}` — **no `dev` field** at
//!   v1-beta (design S1; a `dev` key REJECTS at decode).
//! - `v = 1`, `sig_cp = 0x0001` (LAMPS `id-MLDSA65-Ed25519-SHA512`),
//!   `kem_cp = 0x647a` (`HYBRID_X25519_MLKEM768`).
//! - `kem` multikey is **X25519-first** (design C2, matches the already-frozen
//!   `RecipientPublic::to_bytes` order — no reorder): `0xec01 ‖ x25519(32) ‖
//!   0x120c ‖ mlkem768_ek(1184)`. (Decoded/cross-checked in
//!   [`crate::did::Did::resolve_kem`].)
//! - Commitment algorithm: BLAKE3-256 over canonical DAG-CBOR →
//!   self-describing CIDv1.
//! - Strict-canonical decode ([`KeySetDocument::from_canonical_bytes`]):
//!   rejects indefinite lengths, non-minimal integers, unsorted / duplicate
//!   keys, extra fields (incl. `dev`), trailing bytes, forward versions, and
//!   over-sized (bounded-decode DoS cap) inputs — design C3 / Row-D-13.

use benten_core::Cid;
use serde::{Deserialize, Serialize};

use crate::errors::DidError;

/// [`KeySetDocument`] v1-beta format version (design §1.2).
pub const KEYSET_DOC_VERSION: u16 = 1;

/// Frozen `sig_cp` — LAMPS `id-MLDSA65-Ed25519-SHA512` (design §1.2).
pub const SIG_CP_LAMPS_MLDSA65_ED25519: u16 = 0x0001;

/// Frozen `kem_cp` — `HYBRID_X25519_MLKEM768` (design §1.2).
pub const KEM_CP_HYBRID_X25519_MLKEM768: u16 = 0x647a;

/// Bounded-decode cap (bytes) for [`KeySetDocument::from_canonical_bytes`]
/// (design DOS-1 / META #629). A key-set doc is a SECOND untrusted input the
/// `did:benten` F2 string cap does not cover — a first-contact / cached /
/// iroh-fetched doc is attacker-influenced, so its strict decode MUST be
/// bounded BEFORE any allocating decode. A legitimate v1-beta hybrid key-set
/// (`sig` ≈ 1988 B + `kem` ≈ 1220 B + CBOR overhead ≈ 3.3 KB) is far under
/// this cap; the cap only rejects input already far larger than any
/// structurally-valid document.
pub const MAX_KEYSET_DOC_BYTES: usize = 8 * 1024;

/// A committed key-set document — the CLOSED 5-field canonical DAG-CBOR map a
/// `did:benten` commits by CID (design §1.2 / S1).
///
/// Fields are private; construct via [`Self::v1_hybrid`] / [`Self::v1_with`]
/// or recover via [`Self::from_canonical_bytes`]. [`Self::to_canonical_bytes`]
/// + [`Self::cid`] are the frozen commitment serialization.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KeySetDocument {
    v: u16,
    sig: Vec<u8>,
    kem: Vec<u8>,
    sig_cp: u16,
    kem_cp: u16,
}

/// Canonical DAG-CBOR **serialization** projection. Fields are declared in
/// **DAG-CBOR canonical key order** (length-first then bytewise): `v`(1) <
/// `kem`(3) < `sig`(3) < `kem_cp`(6) < `sig_cp`(6). `serde_bytes` forces the
/// definite-length byte-string encoding (`0x58.. len ..`) that gives
/// injectivity for free (design §1.2).
#[derive(Serialize)]
struct KeySetWire<'a> {
    v: u16,
    #[serde(with = "serde_bytes")]
    kem: &'a [u8],
    #[serde(with = "serde_bytes")]
    sig: &'a [u8],
    kem_cp: u16,
    sig_cp: u16,
}

/// Owned **deserialization** projection — the strict-canonical decode target.
/// `deny_unknown_fields` rejects ANY extra field (incl. a `dev` key, design
/// S1); serde's derived struct decode rejects duplicate + missing fields.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct KeySetWireOwned {
    v: u16,
    #[serde(with = "serde_bytes")]
    kem: Vec<u8>,
    #[serde(with = "serde_bytes")]
    sig: Vec<u8>,
    kem_cp: u16,
    sig_cp: u16,
}

impl KeySetDocument {
    /// The v1-beta hybrid default: `v = 1`, `sig_cp = 0x0001`,
    /// `kem_cp = 0x647a`. `sig` = the signing multikey
    /// (`0x1211‖mldsa ‖ 0xed‖ed25519`, ML-DSA-first), `kem` = the KEM
    /// multikey, **X25519-first** (`0xec‖x25519 ‖ 0x120c‖mlkem768_ek`, C2).
    #[must_use]
    pub fn v1_hybrid(sig: Vec<u8>, kem: Vec<u8>) -> Self {
        Self {
            v: KEYSET_DOC_VERSION,
            sig,
            kem,
            sig_cp: SIG_CP_LAMPS_MLDSA65_ED25519,
            kem_cp: KEM_CP_HYBRID_X25519_MLKEM768,
        }
    }

    /// Fully-explicit constructor for reject-matrix fixtures (arbitrary
    /// version / codepoints / malformed multikeys). Production callers use
    /// [`Self::v1_hybrid`].
    #[must_use]
    pub fn v1_with(v: u16, sig: Vec<u8>, kem: Vec<u8>, sig_cp: u16, kem_cp: u16) -> Self {
        Self {
            v,
            sig,
            kem,
            sig_cp,
            kem_cp,
        }
    }

    /// Canonical DAG-CBOR bytes — the FROZEN commitment preimage (design
    /// §1.2). Deterministic: the same document always serializes to the same
    /// bytes (definite lengths, canonical key order, minimal integers).
    #[must_use]
    pub fn to_canonical_bytes(&self) -> Vec<u8> {
        serde_ipld_dagcbor::to_vec(&KeySetWire {
            v: self.v,
            kem: &self.kem,
            sig: &self.sig,
            kem_cp: self.kem_cp,
            sig_cp: self.sig_cp,
        })
        .expect("DAG-CBOR encoding of the fixed-shape KeySetDocument cannot fail")
    }

    /// The document's CID: `self_describing_cid(BLAKE3-256(canonical))`
    /// (design §1.2). This CID is what a `did:benten` COMMITS — the
    /// 2nd-preimage binding [`crate::did::Did::resolve_kem`] checks.
    #[must_use]
    pub fn cid(&self) -> Cid {
        let digest = blake3::hash(&self.to_canonical_bytes());
        Cid::from_blake3_digest(*digest.as_bytes())
    }

    /// STRICT-canonical decode (design C3 / Row-D-13) — recover a
    /// [`KeySetDocument`] from canonical DAG-CBOR bytes, fail-closed on ANY
    /// non-canonical / malformed / oversized input.
    ///
    /// Rejects (each fail-closed):
    /// - **Over-sized** input, BEFORE the allocating decode (bounded-decode
    ///   DoS cap, [`MAX_KEYSET_DOC_BYTES`]; design DOS-1 / META #629).
    /// - **Indefinite-length**, **duplicate-key**, **extra-field** (incl. a
    ///   `dev` key, S1), **missing-field**, and **trailing-byte** inputs — at
    ///   the strict DAG-CBOR decode.
    /// - **Non-minimal integers** and **unsorted map keys** the underlying
    ///   reader would otherwise accept — enforced by a decode → re-encode →
    ///   byte-equality check (the input MUST already be in canonical form; NOT
    ///   a raw compare against any committed preimage).
    /// - **Forward / unknown versions** (`v != 1`) — a typed-reject, never a
    ///   best-effort parse.
    ///
    /// # Errors
    ///
    /// [`DidError::KeysetDocTooLong`] / [`DidError::KeysetDocMalformed`] /
    /// [`DidError::KeysetDocNonCanonical`] /
    /// [`DidError::KeysetDocUnsupportedVersion`].
    pub fn from_canonical_bytes(bytes: &[u8]) -> Result<Self, DidError> {
        // (1) Bounded-decode DoS cap — reject an over-sized doc BEFORE the
        //     allocating decode (a declared-length bomb is separately bounded
        //     by the strict decoder's own reservation cap; this cap rejects an
        //     oversized-but-well-formed doc that would otherwise decode).
        if bytes.len() > MAX_KEYSET_DOC_BYTES {
            return Err(DidError::KeysetDocTooLong {
                got: bytes.len(),
                max: MAX_KEYSET_DOC_BYTES,
            });
        }

        // (2) Strict DAG-CBOR decode into the CLOSED 5-field shape.
        //     `serde_ipld_dagcbor::from_slice` rejects indefinite lengths +
        //     trailing bytes; `deny_unknown_fields` rejects extra (incl.
        //     `dev`) fields; serde's derived struct decode rejects duplicate
        //     + missing fields.
        let wire: KeySetWireOwned =
            serde_ipld_dagcbor::from_slice(bytes).map_err(|_| DidError::KeysetDocMalformed)?;

        let doc = Self {
            v: wire.v,
            sig: wire.sig,
            kem: wire.kem,
            sig_cp: wire.sig_cp,
            kem_cp: wire.kem_cp,
        };

        // (3) Strict-canonical enforcement (C3, Row-D-13): the input MUST
        //     already be in canonical form. Re-encode the decoded doc and
        //     require byte-equality — this rejects non-minimal integers +
        //     unsorted keys the lenient reader accepts. (A self-canonical
        //     check, NOT a raw compare against a committed preimage.)
        if doc.to_canonical_bytes() != bytes {
            return Err(DidError::KeysetDocNonCanonical);
        }

        // (4) Forward/unknown-version typed-reject (v1-beta decoder).
        if doc.v != KEYSET_DOC_VERSION {
            return Err(DidError::KeysetDocUnsupportedVersion { version: doc.v });
        }

        Ok(doc)
    }

    /// Format version (`v` field).
    #[must_use]
    pub fn version(&self) -> u16 {
        self.v
    }

    /// Signing multikey (`sig` field): `0x1211‖mldsa ‖ 0xed‖ed25519`.
    #[must_use]
    pub fn sig(&self) -> &[u8] {
        &self.sig
    }

    /// KEM multikey (`kem` field): X25519-first `0xec‖x25519 ‖ 0x120c‖mlkem768_ek`.
    #[must_use]
    pub fn kem(&self) -> &[u8] {
        &self.kem
    }

    /// Signature-suite codepoint (`sig_cp` field).
    #[must_use]
    pub fn sig_cp(&self) -> u16 {
        self.sig_cp
    }

    /// Cipher-suite codepoint (`kem_cp` field).
    #[must_use]
    pub fn kem_cp(&self) -> u16 {
        self.kem_cp
    }
}
