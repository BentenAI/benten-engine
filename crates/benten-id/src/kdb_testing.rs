//! GAP-KDB Shape-B — shared RED-PHASE test fixtures (W0 canary).
//!
//! Ref `3bea1294` design spec `GAP-KDB-B-DESIGN-R1.md` + R2 landscape
//! `GAP-KDB-B-R2-LANDSCAPE.md`. This module is the **W0 canary surface**:
//! the frozen `did:benten` + [`KeySetDocument`] + resolver API SHAPE that
//! every downstream wave (W1 crypto-suite, W2 benten-drop Layer-C, W3
//! Fork-A authority, W4 docs) consumes. It is gated behind
//! `#[cfg(any(test, feature = "testing"))]` so it never reaches a
//! production build, and it is a **library** module (not `tests/`) so
//! `benten-drop` + `benten-engine` red-phase tests can `use
//! benten_id::kdb_testing::*` cross-crate (enable `benten-id`'s `testing`
//! feature in their dev-deps).
//!
//! # RED-PHASE stub-shim discipline (how this becomes real at R5)
//!
//! The real GAP-KDB types do NOT exist at the freeze base
//! (`phase-4-meta-core/r9-base`). This module supplies the **expected API
//! shape** so the red-phase test files COMPILE green at baseline while
//! staying `#[ignore]`d. Two disjoint categories live here:
//!
//! - **Fixture DATA + FROZEN-spec serialization (REAL now).** The
//!   multicodec framing constants, [`KeySetDocument::to_canonical_bytes`]
//!   (canonical DAG-CBOR), [`KeySetDocument::cid`] (BLAKE3-256 →
//!   self-describing CIDv1), and the raw `did:benten` string assembler
//!   [`did_benten_from_payload_for_test`] implement the FROZEN spec
//!   directly (deterministic; the same bytes the R5 encoder must emit).
//!   These give stable golden pins + coupled test scenarios.
//! - **LOGIC-UNDER-TEST (STUB `todo!()` now → real entry at R5).** The
//!   codec encoder [`encode_did_benten`], resolvers [`resolve_signing`] /
//!   [`resolve_kem`] / [`committed_keyset_cid`], strict decode
//!   [`KeySetDocument::from_canonical_bytes`], and
//!   [`RecipientBinding::resolve`] are the GAP-KDB security surface. They
//!   are `todo!()` stubs at R3. At R5 each stub body is replaced by a
//!   delegation to the minted real entry (e.g. `did.resolve_kem(doc)`),
//!   and the red-phase tests un-ignore. Because a `todo!()` panics, no
//!   red-phase test can pass against the stub — the ONLY way each pin goes
//!   green is against a real, non-no-op implementation (substance by
//!   construction).
//!
//! **R5 handoff (single-file swap):** mint the real
//! `benten_id::keyset::KeySetDocument` + `Did::{from_benten_keyset,
//! resolve_signing, resolve_kem, keyset_cid}` + benten-drop
//! `RecipientBinding`; then in THIS file (a) replace the stub
//! `KeySetDocument` with `pub use crate::keyset::KeySetDocument;`, (b)
//! replace each `todo!()` free-fn body with the real-method delegation,
//! (c) drop `for_test` escape hatches that the real sole-constructor
//! subsumes. Test call sites do not change — they only un-ignore.

#![allow(
    // RED-PHASE fixtures: never-constructed stub fields, todo!() stubs,
    // and pedantic doc-nits are expected during the red phase (mirrors the
    // workspace `todo = "allow"` posture). The R5 swap removes the stubs.
    clippy::todo,
    clippy::missing_panics_doc,
    clippy::must_use_candidate,
    clippy::missing_errors_doc,
    dead_code
)]

use benten_core::Cid;
use benten_crypto_suite::CipherSuiteCodepoint;
use benten_crypto_suite::cipher_suite::RecipientPublic;
use benten_crypto_suite::sig;
use serde::Serialize;

use crate::did::{Did, ED25519_MULTICODEC, MLDSA65_PUB_MULTICODEC};
use crate::errors::DidError;

// ─────────────────────────────────────────────────────────────────────────
// FROZEN constants (v1-beta wire — MUST match the R5 production surface).
// ─────────────────────────────────────────────────────────────────────────

/// `did:benten` method prefix incl. the `z` base58btc multibase marker.
/// A distinct method makes "this DID commits my full key-set" lexically
/// un-confusable with a signing-only `did:key` (design §1.1).
pub const DID_BENTEN_PREFIX: &str = "did:benten:z";

/// `did:benten` method identifier (no multibase marker).
pub const DID_BENTEN_METHOD: &str = "did:benten:";

/// Registered `mlkem-768-pub = 0x120c`, unsigned-varint `[0x8c, 0x24]`
/// (design §5 — wires the registered component, retiring the #5-risky
/// private `HYBRID_KEM_MULTICODEC = 0xf0`).
pub const MLKEM768_PUB_MULTICODEC: [u8; 2] = [0x8c, 0x24];

/// Registered `x25519-pub = 0xec`, unsigned-varint `[0xec, 0x01]`.
pub const X25519_PUB_MULTICODEC: [u8; 2] = [0xec, 0x01];

/// [`KeySetDocument`] v1-beta format version (design §1.2).
pub const KEYSET_DOC_VERSION: u16 = 1;

/// Frozen `sig_cp` — LAMPS `id-MLDSA65-Ed25519-SHA512` (design §1.2).
pub const SIG_CP_LAMPS_MLDSA65_ED25519: u16 = 0x0001;

/// Frozen `kem_cp` — `HYBRID_X25519_MLKEM768` (design §1.2).
pub const KEM_CP_HYBRID_X25519_MLKEM768: u16 = 0x647a;

/// The below-PQ-floor classical-only `0x6400` cipher suite. A key-set
/// committing this is HNDL-exposed and MUST be rejected by `resolve_kem`
/// (design C5 — PQ floor).
pub const KEM_CP_CLASSICAL_X25519_FLOOR: u16 = 0x6400;

/// The self-describing Benten CIDv1 header `[0x01, 0x71, 0x1e, 0x20]`
/// (version, dag-cbor codec, blake3 multihash, 32-byte digest length).
/// Re-derived from [`benten_core`] so a header drift there fails these
/// pins rather than a hand-copied literal.
pub const CID_V1_DAGCBOR_BLAKE3_HEADER: [u8; 4] = [
    benten_core::CID_V1,
    benten_core::MULTICODEC_DAG_CBOR,
    benten_core::MULTIHASH_BLAKE3,
    benten_core::BLAKE3_DIGEST_LEN,
];

// ─────────────────────────────────────────────────────────────────────────
// KeySetDocument — canonical DAG-CBOR key-set document (design §1.2).
//
// FROZEN schema (canonical map, sorted keys): `{v, sig, kem, sig_cp,
// kem_cp}`. `to_canonical_bytes` + `cid` are the REAL frozen serialization
// (deterministic); `from_canonical_bytes` is the strict-canonical decode
// LOGIC-UNDER-TEST (stub `todo!()` → real at R5).
// ─────────────────────────────────────────────────────────────────────────

/// A committed key-set document. At v1-beta this is a CLOSED 5-field map
/// (design S1: any extra field — including a `dev` key — REJECTS).
///
/// Fields are private; construct via [`Self::v1_hybrid`] /
/// [`Self::v1_with`]. The stub carries the exact FROZEN public API the R5
/// `benten_id::keyset::KeySetDocument` re-export must satisfy.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KeySetDocument {
    v: u16,
    sig: Vec<u8>,
    kem: Vec<u8>,
    sig_cp: u16,
    kem_cp: u16,
}

/// Canonical DAG-CBOR wire projection. Fields declared in **DAG-CBOR
/// canonical key order** (length-first, then bytewise): `v`(1) < `kem`(3)
/// < `sig`(3) < `kem_cp`(6) < `sig_cp`(6). `serde_bytes` forces the
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

impl KeySetDocument {
    /// The v1-beta hybrid default: `v = 1`, `sig_cp = 0x0001`,
    /// `kem_cp = 0x647a`. `sig` = the signing multikey
    /// (`0x1211‖mldsa ‖ 0xed‖ed25519`), `kem` = the KEM multikey,
    /// **X25519-first** (`0xec‖x25519 ‖ 0x120c‖mlkem768_ek`, design C2).
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
    /// version / codepoints / malformed multikeys).
    pub fn v1_with(v: u16, sig: Vec<u8>, kem: Vec<u8>, sig_cp: u16, kem_cp: u16) -> Self {
        Self {
            v,
            sig,
            kem,
            sig_cp,
            kem_cp,
        }
    }

    /// Canonical DAG-CBOR bytes — the FROZEN commitment preimage
    /// (design §1.2). REAL; the R5 encoder MUST produce identical bytes
    /// (pinned by KSD-1).
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
    /// (design §1.2). This CID is what a `did:benten` COMMITS. REAL.
    pub fn cid(&self) -> Cid {
        let digest = blake3::hash(&self.to_canonical_bytes());
        Cid::from_blake3_digest(*digest.as_bytes())
    }

    /// STRICT-canonical decode (design C3 / Row-D-13) — the
    /// LOGIC-UNDER-TEST for KSD-3/5/8 + DOS-1. Rejects indefinite-length
    /// / duplicate-key / unsorted-key / non-minimal-int / trailing /
    /// extra-key (incl. `dev`) / forward-version / bounded-decode-cap.
    ///
    /// R5: replace this stub with the real strict decoder (or re-export
    /// `benten_id::keyset::KeySetDocument::from_canonical_bytes`).
    pub fn from_canonical_bytes(_bytes: &[u8]) -> Result<Self, DidError> {
        todo!(
            "RED-PHASE (KSD-3/5/8, DOS-1): strict-canonical KeySetDocument decode \
             lands at R5 (GAP-KDB-B canary). un-ignore then."
        )
    }

    /// Format version (`v` field).
    pub fn version(&self) -> u16 {
        self.v
    }
    /// Signing multikey (`sig` field).
    pub fn sig(&self) -> &[u8] {
        &self.sig
    }
    /// KEM multikey (`kem` field).
    pub fn kem(&self) -> &[u8] {
        &self.kem
    }
    /// Signature-suite codepoint (`sig_cp` field).
    pub fn sig_cp(&self) -> u16 {
        self.sig_cp
    }
    /// Cipher-suite codepoint (`kem_cp` field).
    pub fn kem_cp(&self) -> u16 {
        self.kem_cp
    }
}

// ─────────────────────────────────────────────────────────────────────────
// RecipientBinding — the seal-API typestate (design §5).
//
// The REAL type lands in benten-drop Layer-C at R5 (W2). This stub exists
// only so cross-crate red-phase tests can name the SHAPE + so the sole-
// constructor / no-fallback-door discipline (design C4) has a pin surface.
// ─────────────────────────────────────────────────────────────────────────

/// A recipient whose KEM key is PROVEN committed by its DID (design §5 —
/// Inv-23). The real type + its sole `resolve` constructor land in
/// benten-drop at R5; this stub mirrors the frozen shape for cross-crate
/// W2 red-phase tests.
///
/// (No `Debug` derive: `RecipientPublic` carries no `Debug` impl — key
/// material stays out of any `Debug` sink.)
pub struct RecipientBinding {
    audience_did: Did,
    kem_pub: RecipientPublic,
}

impl RecipientBinding {
    /// The ONLY real constructor (design C4). Fail-closed typed-reject on
    /// commitment mismatch — calls `resolve_kem` internally. STUB
    /// `todo!()` → real at R5 (benten-drop Layer-C).
    pub fn resolve(_audience_did: &Did, _keyset_doc: &KeySetDocument) -> Result<Self, DidError> {
        todo!(
            "RED-PHASE (DROP-2): RecipientBinding::resolve (sole constructor, \
             no fallback door — design C4) lands in benten-drop Layer-C at R5 (W2)."
        )
    }

    /// Fixture escape hatch — pairs an ARBITRARY (possibly un-committed)
    /// KEM key with a DID, bypassing the commitment check. Used ONLY to
    /// stage the anti-downgrade / substitution scenarios W2 exercises;
    /// the real API has NO such door (design C4). Openly `_for_test`.
    pub fn for_test_unchecked(audience_did: Did, kem_pub: RecipientPublic) -> Self {
        Self {
            audience_did,
            kem_pub,
        }
    }

    /// The bound audience DID.
    pub fn audience_did(&self) -> &Did {
        &self.audience_did
    }
    /// The committed KEM key.
    pub fn kem_pub(&self) -> &RecipientPublic {
        &self.kem_pub
    }
}

// ─────────────────────────────────────────────────────────────────────────
// CODEC + RESOLVERS — LOGIC-UNDER-TEST (stub `todo!()` → real entry at R5).
//
// Free functions so the R3→R5 swap is a single-file edit and the test
// call sites never change. At R5 each body delegates to the minted real
// method (noted per fn).
// ─────────────────────────────────────────────────────────────────────────

/// Encode a `did:benten` committing `doc` (design §1.1): method-specific
/// id = `signing_multikey(sig_pk) ‖ doc.cid()` (36 B), base58btc, no
/// framing byte (design C1). STUB → R5 `Did::from_benten_keyset(sig_pk, doc)`.
pub fn encode_did_benten(_sig_pk: &sig::PublicKey, _doc: &KeySetDocument) -> Did {
    todo!(
        "RED-PHASE (DID-1/2/6): Did::from_benten_keyset (did:benten encoder) \
         lands at R5 (GAP-KDB-B canary). un-ignore then."
    )
}

/// Method-aware signing-key resolve (design §2 Tier-1, C6). `did:key`
/// (0xed01) → classical; hybrid `did:key` (0x1211) → composite;
/// `did:benten` → composite + strip the trailing keyset-CID component.
/// Zero-I/O, no doc. STUB → R5 `did.resolve_signing()`.
pub fn resolve_signing(_did: &Did) -> Result<sig::PublicKey, DidError> {
    todo!(
        "RED-PHASE (DID-4, RS-1/2): Did::resolve_signing (method/multicodec-aware) \
         lands at R5 (GAP-KDB-B canary). un-ignore then."
    )
}

/// KEM-key resolve (design §2 Tier-2). Recovers + VERIFIES the recipient
/// KEM key from the DID's key-set commitment: (1) `cid(doc) ==
/// committed_cid` (2nd-preimage), (2) `doc.sig == embedded_signing`
/// cross-check, (3) `kem_cp ⟺ components`, (4) PQ-floor (`0x6400`
/// reject). Fail-closed on ANY mismatch. STUB → R5 `did.resolve_kem(doc)`.
pub fn resolve_kem(_did: &Did, _doc: &KeySetDocument) -> Result<RecipientPublic, DidError> {
    todo!(
        "RED-PHASE (RK-1..7, flagship RK-2): Did::resolve_kem (CID 2nd-preimage \
         fail-closed) lands at R5 (GAP-KDB-B canary). un-ignore then."
    )
}

/// The committed key-set CID carried in a `did:benten` string (the last
/// 36 payload bytes). A bare `did:key` commits none →
/// `NoKemCommitment`-class reject (design §6). STUB → R5
/// `did.keyset_cid()`.
pub fn committed_keyset_cid(_did: &Did) -> Result<Cid, DidError> {
    todo!(
        "RED-PHASE (RK-7, DID-3): Did::keyset_cid accessor lands at R5 \
         (GAP-KDB-B canary). un-ignore then."
    )
}

// ─────────────────────────────────────────────────────────────────────────
// FIXTURE BUILDERS (REAL) — construct frozen-spec inputs + attack scenarios.
// ─────────────────────────────────────────────────────────────────────────

/// A real hybrid (Ed25519⊕ML-DSA-65) keypair via the v1-beta default
/// suite. Random (ML-DSA keygen is non-deterministic); use for
/// round-trip / resolve tests whose assertions are recovery-equality,
/// not frozen hex.
pub fn hybrid_keypair() -> sig::Keypair {
    sig::SignatureSuite::v1_default().generate_keypair()
}

/// The signing multikey a `did:benten` embeds (and a [`KeySetDocument`]'s
/// `sig` field carries): `0x1211‖mldsaPK(1952) ‖ 0xed‖ed25519PK(32)`,
/// ML-DSA-FIRST (byte-identical to the hybrid `did:key` payload). REAL
/// frozen assembly.
pub fn signing_multikey_of(pk: &sig::PublicKey) -> Vec<u8> {
    let composite = pk
        .to_lamps_composite_bytes()
        .expect("hybrid public key required for a signing multikey");
    let mldsa_len = benten_crypto_suite::sizes::ml_dsa_65_pubkey_len();
    let (mldsa_pk, trad_pk) = composite.split_at(mldsa_len);
    let mut out = Vec::with_capacity(2 + mldsa_pk.len() + 2 + trad_pk.len());
    out.extend_from_slice(&MLDSA65_PUB_MULTICODEC);
    out.extend_from_slice(mldsa_pk);
    out.extend_from_slice(&ED25519_MULTICODEC);
    out.extend_from_slice(trad_pk);
    out
}

/// The hybrid KEM multikey — **X25519-first** (design C2, no reorder):
/// `0xec‖x25519(32) ‖ 0x120c‖mlkem768_ek(1184)`. REAL frozen assembly.
pub fn kem_multikey_hybrid(x25519_pub: &[u8; 32], mlkem768_ek: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(2 + 32 + 2 + mlkem768_ek.len());
    out.extend_from_slice(&X25519_PUB_MULTICODEC);
    out.extend_from_slice(x25519_pub);
    out.extend_from_slice(&MLKEM768_PUB_MULTICODEC);
    out.extend_from_slice(mlkem768_ek);
    out
}

/// The classical-only (below-PQ-floor) KEM multikey: `0xec‖x25519(32)`.
/// A key-set carrying only this commits `0x6400` and MUST be rejected by
/// `resolve_kem` (design C5).
pub fn kem_multikey_classical(x25519_pub: &[u8; 32]) -> Vec<u8> {
    let mut out = Vec::with_capacity(2 + 32);
    out.extend_from_slice(&X25519_PUB_MULTICODEC);
    out.extend_from_slice(x25519_pub);
    out
}

/// Deterministic, reproducible bytes (BLAKE3 XOF fill). Used for stable
/// golden fixtures where key VALIDITY is irrelevant (the codec frames
/// opaque bytes) — never fed to a key-validating path.
pub fn det_bytes(seed: &str, len: usize) -> Vec<u8> {
    let mut out = vec![0u8; len];
    let mut xof = blake3::Hasher::new();
    xof.update(seed.as_bytes());
    xof.finalize_xof().fill(&mut out);
    out
}

/// A deterministic 32-byte X25519-public placeholder.
pub fn det_x25519_pub(seed: &str) -> [u8; 32] {
    let v = det_bytes(seed, 32);
    let mut a = [0u8; 32];
    a.copy_from_slice(&v);
    a
}

/// A deterministic 1184-byte ML-KEM-768 EK placeholder (opaque; framing
/// fixture only).
pub fn det_mlkem768_ek(seed: &str) -> Vec<u8> {
    det_bytes(seed, benten_crypto_suite::cipher_suite::ML_KEM_768_EK_LEN)
}

/// A deterministic signing multikey with the FROZEN framing over opaque
/// (`det_bytes`) component payloads — for golden-LAYOUT pins that must be
/// byte-stable (KSD-1 / DID-1 golden-hex capture).
pub fn det_signing_multikey(seed: &str) -> Vec<u8> {
    let mldsa = det_bytes(
        &format!("{seed}/mldsa"),
        benten_crypto_suite::sizes::ml_dsa_65_pubkey_len(),
    );
    let ed = det_bytes(&format!("{seed}/ed25519"), 32);
    let mut out = Vec::new();
    out.extend_from_slice(&MLDSA65_PUB_MULTICODEC);
    out.extend_from_slice(&mldsa);
    out.extend_from_slice(&ED25519_MULTICODEC);
    out.extend_from_slice(&ed);
    out
}

/// Assemble a `did:benten` string from a RAW method-specific-id payload —
/// the fixture escape hatch for crafting attack DIDs (mismatched
/// embedded-sig vs committed-CID for RK-3; trailing-byte / truncation
/// injection for DID-3; CID-sensitivity for DID-6). REAL base58btc
/// assembly following the frozen §1.1 layout; NOT the production encoder
/// (that is [`encode_did_benten`], the logic-under-test). Openly
/// `_for_test`.
pub fn did_benten_from_payload_for_test(payload: &[u8]) -> Did {
    let body = bs58::encode(payload).into_string();
    Did::from_string_for_test_fixture(format!("{DID_BENTEN_PREFIX}{body}"))
}

/// The canonical `did:benten` payload for a `(signing multikey, keyset
/// CID)` pair per the frozen §1.1 layout: `signing_multikey ‖
/// cid(36)`, no framing byte (design C1).
pub fn did_benten_payload(signing_multikey: &[u8], keyset_cid: &Cid) -> Vec<u8> {
    let mut payload = Vec::with_capacity(signing_multikey.len() + benten_core::CID_LEN);
    payload.extend_from_slice(signing_multikey);
    payload.extend_from_slice(keyset_cid.as_bytes());
    payload
}

/// Base58btc-decode a `did:benten` string back to its raw method-specific
/// -id payload bytes (the inverse of [`did_benten_from_payload_for_test`]).
/// `None` if the DID is not a `did:benten` or the body fails base58 decode.
pub fn did_benten_payload_of(did: &Did) -> Option<Vec<u8>> {
    let body = did.as_str().strip_prefix(DID_BENTEN_PREFIX)?;
    bs58::decode(body).into_vec().ok()
}

/// The HONEST coupled scenario: a `did:benten` whose committed CID is
/// exactly `cid(doc)` (built via the frozen layout, since the production
/// encoder is a stub at R3). Returns `(audience_did, doc)`.
pub fn honest_recipient_scenario() -> (Did, KeySetDocument) {
    let kp = hybrid_keypair();
    let sig_mk = signing_multikey_of(&kp.public());
    let kem_mk = kem_multikey_hybrid(
        &det_x25519_pub("honest/x25519"),
        &det_mlkem768_ek("honest/mlkem"),
    );
    let doc = KeySetDocument::v1_hybrid(sig_mk.clone(), kem_mk);
    let payload = did_benten_payload(&sig_mk, &doc.cid());
    let did = did_benten_from_payload_for_test(&payload);
    (did, doc)
}

/// A `did:benten` that self-commits `doc`: its embedded signing multikey
/// is exactly `doc.sig()` and its committed CID is exactly `cid(doc)`
/// (built via the frozen §1.1 layout). Used to isolate a SPECIFIC
/// `resolve_kem` reject arm (bad kem-multikey / below-floor / etc.) with
/// steps 1 (CID) and 2 (`doc.sig == embedded`) already satisfied — so the
/// reject can only come from the arm under test.
pub fn self_committed_did(doc: &KeySetDocument) -> Did {
    let payload = did_benten_payload(doc.sig(), &doc.cid());
    did_benten_from_payload_for_test(&payload)
}

/// The GAP-KDB active-substitution scenario (RK-2 / DROP-1 flagship
/// inputs). The victim `did:benten` commits `cid(honest_doc)`; the
/// returned `attacker_doc` carries an ATTACKER-controlled KEM key and (by
/// 2nd-preimage resistance) a DIFFERENT canonical CID. `resolve_kem(&
/// victim_did, &attacker_doc)` MUST fail closed. Returns `(victim_did,
/// honest_doc, attacker_doc)`.
pub fn substituted_recipient_scenario() -> (Did, KeySetDocument, KeySetDocument) {
    let kp = hybrid_keypair();
    let sig_mk = signing_multikey_of(&kp.public());

    let honest_kem = kem_multikey_hybrid(
        &det_x25519_pub("victim/x25519"),
        &det_mlkem768_ek("victim/mlkem"),
    );
    let honest_doc = KeySetDocument::v1_hybrid(sig_mk.clone(), honest_kem);

    // Attacker swaps in their own KEM key. Same embedded signing key, so
    // the ONLY thing that changes is the committed key-set CID.
    let attacker_kem = kem_multikey_hybrid(
        &det_x25519_pub("attacker/x25519"),
        &det_mlkem768_ek("attacker/mlkem"),
    );
    let attacker_doc = KeySetDocument::v1_hybrid(sig_mk.clone(), attacker_kem);

    // The victim DID commits the HONEST doc's CID (frozen layout).
    let payload = did_benten_payload(&sig_mk, &honest_doc.cid());
    let victim_did = did_benten_from_payload_for_test(&payload);
    (victim_did, honest_doc, attacker_doc)
}

// ─────────────────────────────────────────────────────────────────────────
// M-20 golden capture helper.
// ─────────────────────────────────────────────────────────────────────────

/// Lower-hex encode (hand-rolled; avoids a dep for a `feature="testing"`
/// lib module). The M-20 discipline: at R5, run a THROWAWAY that
/// `golden_hex(real_encoder_output())`, freeze the returned string as a
/// test constant, then compare the production encoder against it. Never
/// hand-author a golden (a hand golden matching a buggy encoder freezes
/// the bug).
pub fn golden_hex(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut s = String::with_capacity(bytes.len() * 2);
    for &b in bytes {
        s.push(HEX[(b >> 4) as usize] as char);
        s.push(HEX[(b & 0x0f) as usize] as char);
    }
    s
}

/// Decode a lower-hex golden string back to bytes (inverse of
/// [`golden_hex`]). Panics on odd length / non-hex — golden constants are
/// author-controlled.
pub fn from_golden_hex(s: &str) -> Vec<u8> {
    fn nib(c: u8) -> u8 {
        match c {
            b'0'..=b'9' => c - b'0',
            b'a'..=b'f' => c - b'a' + 10,
            b'A'..=b'F' => c - b'A' + 10,
            _ => panic!("non-hex character in golden constant"),
        }
    }
    let b = s.as_bytes();
    assert!(b.len().is_multiple_of(2), "golden hex must be even length");
    b.chunks_exact(2)
        .map(|p| (nib(p[0]) << 4) | nib(p[1]))
        .collect()
}

/// Convenience: the live `HYBRID_X25519_MLKEM768` cipher-suite codepoint
/// handle (`0x647a`) for scenario construction.
pub fn hybrid_kem_codepoint() -> CipherSuiteCodepoint {
    CipherSuiteCodepoint::HYBRID_X25519_MLKEM768
}
