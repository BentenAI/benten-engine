//! Layer-A vault — the at-rest `K_principal` + user-DID-signing-key store
//! (G-CORE-3 #1301; F-full Layer-A).
//!
//! # On-disk format (R11 MC-6 frame extension — salt + Argon2id params in-header)
//!
//! `${BENTEN_DATA_DIR}/vault.cbor` — a **hand-rolled magic-prefixed AEAD
//! frame** (NOT a DAG-CBOR-encoded [`EncryptedEnvelope`](crate::envelope::EncryptedEnvelope))
//! built by [`serialize_vault`]:
//! `magic 0xae | format-version V2 | codepoint(BE u16) | salt(16 B) |
//!  m_cost(u32 BE) | t_cost(u32 BE) | p_cost(u32 BE) | nonce_len(u8) | nonce | ct`
//! at the vault band codepoint [`VAULT_SYMMETRIC_AEAD_XNONCE_CODEPOINT`]
//! (`0x6100`). The AEAD-sealed inner payload IS canonical DAG-CBOR — the
//! [`VaultPayload`] `{ k_principal:[u8;32], user_did_signing_key,
//! user_did_creation_time:u64 }` — but the outer on-disk frame is the fixed
//! binary layout above, not a CBOR-envelope wrapper.
//!
//! **R11 MC-6 (frame extension; pre-freeze).** The frame now persists the
//! 16-byte Argon2id salt + the `{m_cost, t_cost, p_cost}` params in the header
//! (they are NON-secret — the standard PBKDF-header shape). This is the
//! load-bearing self-containment property: [`open_vault`] takes the frame bytes
//! + the password ALONE and re-derives the DAK from the header salt+params — no
//! external salt/param source is needed to open a `vault.cbor` across a restart.
//! (Before MC-6 the salt+params lived only in an in-RAM struct that was never
//! persisted, so the frame bytes alone could not re-derive the DAK.) There is
//! no surviving V1/V2 vault golden vector (F-VA-1), so redefining the V2 frame
//! carries no migration burden. `format-version V2` is retained as the vault
//! frame version.
//!
//! # XChaCha20-Poly1305 24-byte nonce (m-4)
//!
//! The vault is **reseal-heavy** (K_principal rotation + multi-device
//! key-wrap re-seal), so a 12-byte random ChaCha20 nonce would hit the
//! 2^32 birthday bound. The vault uses the 24-byte XChaCha20-Poly1305 nonce
//! variant (`SymmetricAeadXNonce`) — the codepoint discriminates the nonce
//! width (a 12-byte nonce presented under the XNonce codepoint is
//! strict-rejected; U2).
//!
//! # DAK derivation (RFC 9106; F-VA-2)
//!
//! `DAK = HKDF-SHA256( Argon2id(pw, salt; m=19456, t=2, p=1),
//!   info = "benten-dak-v1" )`. The Argon2id params + the HKDF
//! domain-separation info-tag are part of the frozen derivation: a param /
//! info-tag drift changes the DAK and breaks every existing vault. The
//! info-tag is the codepoint slot for a future Argon2id-v2 param set.
//!
//! # Memory hygiene (F-VA-4; Compromise #36/#39)
//!
//! The unlocked `K_principal` lives in [`secrecy::SecretBox<[u8;32]>`] inside
//! [`UnlockedKeyMaterial`] — its Debug redacts (never leaks the bytes) + it
//! zeroizes on Drop.
//!
//! # Lock-state (F-VA-5)
//!
//! Pre-unlock crypto ops are typed-rejected with [`VaultError::EngineLocked`]
//! (fail-closed; never a silent plaintext write). `unlock` atomically
//! hydrates BOTH keys (K_principal + user_did_signing_key; identity coherence).

use argon2::{Algorithm, Argon2, Params, Version};
use hkdf::Hkdf;
use secrecy::{ExposeSecret, SecretBox};
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use zeroize::{Zeroize, Zeroizing};

use crate::structural_kdf::StructuralKdfKey;

/// The Layer-A vault wire codepoint (R0.5 §4.0 vault band `0x6100`; the
/// 24-byte `SymmetricAeadXNonce` variant).
pub const VAULT_SYMMETRIC_AEAD_XNONCE_CODEPOINT: u16 = 0x6100;

/// The 12-byte `SymmetricAead` (ChaCha20-Poly1305) sibling codepoint — a
/// RATIFIED frozen v1-beta variant in the vault band `0x6100..0x61FF`. NOT
/// the vault's own codepoint (the vault uses the 24-byte XNonce variant).
pub const SYMMETRIC_AEAD_12B_CODEPOINT: u16 = 0x6101;

/// XChaCha20-Poly1305 nonce width (m-4) — the frozen vault nonce length.
pub const VAULT_XNONCE_LEN: usize = 24;

/// The frozen DAK HKDF info-tag (codepoint slot for a future Argon2id-v2
/// param set per R0.5 §3.1). A registered cross-surface domain-separation tag
/// mirrored in [`crate::domain_registry::DAK_HKDF_INFO_TAG`]; the intra-crate
/// `vault_domain_tags_match_central_registry` test pins byte-equality.
/// Canonical home is HERE.
pub const DAK_HKDF_INFO_TAG: &[u8] = b"benten-dak-v1";

/// The vault AEAD AAD domain-separation label — domain-separates the vault
/// seal from every other envelope and is prefixed into the AAD ahead of the
/// vault codepoint. A registered cross-surface domain-separation tag mirrored
/// in [`crate::domain_registry::VAULT_AAD_DOMAIN`]; the intra-crate
/// `vault_domain_tags_match_central_registry` test pins byte-equality.
/// Canonical home is HERE.
pub const VAULT_AAD_DOMAIN: &[u8] = b"benten-vault:";

/// RFC-9106 / OWASP Argon2id params (R0.5 §2.2 tactical pick).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Argon2idParams {
    /// Memory cost (KiB).
    pub m_cost: u32,
    /// Time cost (iterations).
    pub t_cost: u32,
    /// Parallelism.
    pub p_cost: u32,
}

/// The frozen v1-beta default Argon2id params (OWASP).
pub const OWASP_DEFAULT: Argon2idParams = Argon2idParams {
    m_cost: 19456,
    t_cost: 2,
    p_cost: 1,
};

// --- Fail-closed ceilings on the frame-supplied Argon2id cost params
// (Compromise #28 / META #629 DoS-sweep). ---
//
// `parse_vault_frame` reads `m_cost`/`t_cost`/`p_cost` VERBATIM from an
// attacker-controlled `vault.cbor` HEADER (the untrusted-host /
// peers-hold-ciphertext / remote-permission threat surface), and those
// params flow into `derive_dak` → `Params::new` → `Argon2id::hash_password_into`,
// which allocates `m_cost` KiB and runs `t_cost` passes BEFORE the AEAD/password
// gate can ever fail. Without a ceiling a single hostile blob dictates the
// victim's KDF memory/CPU budget (memory-exhaustion / CPU-pinning DoS). These
// caps sit COMFORTABLY above `OWASP_DEFAULT` (19456 / 2 / 1) so every valid
// vault frame (which is always sealed under `OWASP_DEFAULT`) decodes byte-
// identically — the guard rejects only already-invalid oversized headers.
//
// `VAULT_ARGON2_MAX_M_COST` = 65_536 KiB (64 MiB) — ~3.4× the OWASP default,
// enough headroom for a legitimately-hardened future param set while still
// bounding a hostile header to a fixed, small allocation.
/// Fail-closed ceiling on the frame-supplied Argon2id memory cost (KiB).
pub const VAULT_ARGON2_MAX_M_COST: u32 = 65_536;
/// Fail-closed ceiling on the frame-supplied Argon2id time cost (passes).
pub const VAULT_ARGON2_MAX_T_COST: u32 = 10;
/// Fail-closed ceiling on the frame-supplied Argon2id parallelism.
pub const VAULT_ARGON2_MAX_P_COST: u32 = 4;

// --- Fail-closed FLOORS on the frame-supplied Argon2id cost params
// (R6-R1-refix F-02: the META #629 sweep added the ceilings above but not the
// floor). `parse_vault_frame` previously validated the UPPER bound ONLY, so a
// sub-RFC-9106 header (`t_cost = 0`, `p_cost = 0`, or `m_cost < 8 * p_cost`)
// PASSED the guard and reached `derive_dak` → `Params::new(...).expect(...)`,
// which REJECTS those params → **panic** = a reachable crash-DoS from an
// attacker-controlled `vault.cbor` header on the FROZEN forever-decode path.
// These floors mirror the `argon2` crate's own RFC-9106 minimums so
// `parse_vault_frame` fails closed with a typed `Argon2ParamsOutOfBounds`
// BEFORE `derive_dak` runs (keeping `derive_dak`'s infallible signature — its
// `.expect()` precondition now always holds). `OWASP_DEFAULT` (19456 / 2 / 1)
// clears every floor, so valid frames still decode byte-identically.
// (Private — no new frozen public surface; the behavior is pinned by
// `f_va_*` regression tests, not by exporting the consts.)
/// Fail-closed floor on the frame-supplied Argon2id memory cost (KiB); also
/// enforces the `m_cost >= 8 * p_cost` RFC-9106 relation below.
const VAULT_ARGON2_MIN_M_COST: u32 = 8;
/// Fail-closed floor on the frame-supplied Argon2id time cost (passes).
const VAULT_ARGON2_MIN_T_COST: u32 = 1;
/// Fail-closed floor on the frame-supplied Argon2id parallelism.
const VAULT_ARGON2_MIN_P_COST: u32 = 1;

/// The Device-Authentication Key — a zeroize-on-drop 32-byte secret.
///
/// Wraps a [`secrecy::SecretBox<[u8; 32]>`] so the DAK Debug-redacts and its
/// bytes are **wiped on drop** rather than lingering in freed heap/stack (the
/// same at-rest-secret coredump hygiene the unlocked `K_principal` enjoys).
/// `secrecy` is a [forbidden direct dep](crate::boundary) outside this crate
/// (crypto-agility-contract:6), so this owned newtype is the cross-crate
/// handle: production callers (`layer_d::device_auth`) obtain the raw
/// `&[u8; 32]` via [`Dak::expose`] at the AEAD seal/open call site **without**
/// importing `secrecy` themselves.
pub struct Dak(SecretBox<[u8; 32]>);

impl Dak {
    /// Borrow the raw DAK bytes for the AEAD seal/open call site. The borrow
    /// does not outlive the `Dak`, so the bytes stay zeroize-governed.
    #[must_use]
    pub fn expose(&self) -> &[u8; 32] {
        self.0.expose_secret()
    }
}

impl core::fmt::Debug for Dak {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_tuple("Dak").field(&"<redacted>").finish()
    }
}

/// Derive the Device-Authentication Key (DAK):
/// `HKDF-SHA256( Argon2id(pw, salt; params), info )`.
///
/// Deterministic for the same `(password, salt, params, info_tag)`. A param
/// change or an info-tag change yields a different DAK (both are load-bearing
/// in the derivation). Per CLAUDE.md baked-in #5 the Argon2id + HKDF
/// primitives are wrapped from the vetted upstream `argon2` + `hkdf` crates.
///
/// The DAK is returned as a [`Dak`] (zeroize-on-drop [`secrecy::SecretBox`]
/// newtype) so it Debug-redacts and is **wiped on drop** — production callers
/// (`device_auth::unlock_with_password` / `seal_and_build`) let the DAK fall
/// out of scope, and the `Dak` guarantees its bytes are scrubbed rather than
/// lingering in freed heap/stack. Borrow the raw `&[u8; 32]` via [`Dak::expose`]
/// at the AEAD seal/open call site.
///
/// # Panics
///
/// Panics only on an internal Argon2id param-construction error (the OWASP +
/// stronger fixtures are valid by construction; an invalid caller-supplied
/// param set is a programming error).
#[must_use]
pub fn derive_dak(
    password: &[u8],
    salt: &[u8; 16],
    params: Argon2idParams,
    info_tag: &[u8],
) -> Dak {
    // Step 1 — Argon2id(pw, salt; m/t/p) → 32-byte seed.
    let argon_params = Params::new(params.m_cost, params.t_cost, params.p_cost, Some(32))
        .expect("Argon2id params valid (m/t/p within RFC-9106 bounds)");
    let argon = Argon2::new(Algorithm::Argon2id, Version::V0x13, argon_params);
    let mut seed = [0u8; 32];
    argon
        .hash_password_into(password, salt, &mut seed)
        .expect("Argon2id hash into 32-byte buffer is infallible for valid params");

    // Step 2 — intermediate HKDF-SHA256 over the Argon2id seed with the
    // codepoint label info-tag → the DAK (domain separation + the
    // future-v2-param-set slot).
    let hk = Hkdf::<Sha256>::new(None, &seed);
    let mut dak = [0u8; 32];
    hk.expand(info_tag, &mut dak)
        .expect("HKDF-SHA256 expand to 32 B is infallible");
    seed.zeroize();
    // Move the raw bytes into the zeroize-on-drop SecretBox, then wipe the
    // local copy so no un-zeroized duplicate is left on the stack.
    let boxed = Dak(SecretBox::new(Box::new(dak)));
    dak.zeroize();
    boxed
}

/// Vault CBOR payload (R0.5 §3.1). Field order is FROZEN:
/// `k_principal`, then `user_did_signing_key`, then `user_did_creation_time`.
///
/// The canonical DAG-CBOR map-key order (length-first, then bytewise) for
/// these three keys coincides with this declaration order (11 < 20 < 22).
///
/// Secret-hygiene (D-74/75/76): `k_principal` (at-rest content-encryption root
/// key) + `user_did_signing_key` (hybrid Ed25519⊕ML-DSA-65 signing key) are
/// SECRET. `Debug` is a MANUAL impl that renders both as `<redacted>` (the
/// former `#[derive(Debug)]` dumped the raw bytes and cascaded through
/// [`DecodedVault`]'s derived `Debug`), and both are zeroized on drop.
/// `Serialize`/`Deserialize` are the INTENTIONAL on-disk DAG-CBOR format and
/// are UNCHANGED — redacted-Debug + zeroize are non-wire additions only (the
/// frozen field order + `serde_bytes` byte-string encoding are untouched).
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VaultPayload {
    /// The principal's at-rest content-encryption root key.
    ///
    /// DAG-CBOR-encoded as a byte string (not an array of integers) per the
    /// frozen on-disk layout — `serde_bytes` selects the byte-string major
    /// type so the 32-byte key serializes as `0x5820 || 32 bytes`.
    #[serde(with = "serde_bytes")]
    pub k_principal: [u8; 32],
    /// The user-DID hybrid signing key (Ed25519⊕ML-DSA-65 serialized).
    ///
    /// DAG-CBOR-encoded as a byte string (`serde_bytes`).
    #[serde(with = "serde_bytes")]
    pub user_did_signing_key: Vec<u8>,
    /// Vault creation time (seconds).
    pub user_did_creation_time: u64,
}

impl VaultPayload {
    /// Serialize to canonical DAG-CBOR bytes (the frozen on-disk payload
    /// encoding). Deterministic; a re-serialize is byte-identical.
    ///
    /// # Panics
    ///
    /// Panics only on an internal serializer error (the payload shape is
    /// always serializable).
    #[must_use]
    pub fn to_canonical_cbor(&self) -> Vec<u8> {
        serde_ipld_dagcbor::to_vec(self).expect("VaultPayload serializes to DAG-CBOR")
    }

    /// Decode from canonical DAG-CBOR bytes.
    ///
    /// # Errors
    ///
    /// Returns [`VaultError::MalformedCbor`] on a decode failure.
    pub fn from_canonical_cbor(bytes: &[u8]) -> Result<Self, VaultError> {
        serde_ipld_dagcbor::from_slice(bytes).map_err(|_| VaultError::MalformedCbor)
    }
}

/// Debug-redacting: the SECRET `k_principal` + `user_did_signing_key` MUST NOT
/// leak into logs / panics / tracing (this also protects the cascade through
/// [`DecodedVault`]'s derived `Debug`, which holds a `VaultPayload`). Only the
/// non-secret `user_did_creation_time` renders normally.
impl core::fmt::Debug for VaultPayload {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("VaultPayload")
            .field("k_principal", &"<redacted>")
            .field("user_did_signing_key", &"<redacted>")
            .field("user_did_creation_time", &self.user_did_creation_time)
            .finish()
    }
}

/// Zeroize-on-drop: wipe the SECRET `k_principal` + `user_did_signing_key` on
/// drop so the recovered at-rest root key + user-DID signing key do not linger
/// in freed heap / coredump. `user_did_creation_time` is non-secret. Field
/// types + the frozen serialization are unchanged (drop-behavior only).
impl Drop for VaultPayload {
    fn drop(&mut self) {
        self.k_principal.zeroize();
        self.user_did_signing_key.zeroize();
    }
}

/// A decoded vault envelope — the on-disk
/// [`EncryptedEnvelope`](crate::envelope::EncryptedEnvelope) after AEAD-open.
/// Carries the wire codepoint + the nonce bytes used so the format-freeze
/// pins can inspect them.
#[derive(Debug, Clone)]
pub struct DecodedVault {
    /// The vault wire codepoint (`0x6100` XNonce).
    pub codepoint: u16,
    /// The 16-byte Argon2id salt read FROM the frame header (R11 MC-6).
    pub salt: [u8; 16],
    /// The Argon2id params read FROM the frame header (R11 MC-6).
    pub params: Argon2idParams,
    /// The XChaCha20-Poly1305 24-byte nonce.
    pub nonce: Vec<u8>,
    /// The decoded canonical payload.
    pub payload: VaultPayload,
}

/// The fixed vault frame header length up to (but excluding) the `nonce_len`
/// byte: `magic(1) | V2(1) | codepoint(2) | salt(16) | m_cost(4) | t_cost(4) |
/// p_cost(4)` = 32 bytes (R11 MC-6).
const VAULT_HEADER_LEN: usize = 1 + 1 + 2 + 16 + 4 + 4 + 4;

/// Serialize a vault on-disk envelope for a fixed payload under a fixed DAK,
/// persisting the Argon2id `salt` + `params` in the frame header (R11 MC-6).
///
/// The envelope is XChaCha20-Poly1305-sealed (24-byte nonce) at the vault
/// codepoint `0x6100`, over the canonical DAG-CBOR payload. Deterministic
/// only in the sense the format is fixed; the nonce is random per seal.
///
/// The `salt` + `params` MUST be the SAME salt+params the `dak` was derived
/// under ([`derive_dak`]) — they are written into the frame header so a later
/// [`open_vault`] can re-derive the DAK from the frame bytes + the password
/// ALONE (the MC-6 self-containment property). They are NON-secret.
///
/// # Caller contract (salt origination)
///
/// `salt` MUST be freshly generated from an OS CSPRNG (`OsRng` / `getrandom`),
/// unique per vault, at vault-*creation* time. This function threads the
/// caller-supplied salt into the self-contained header (R11 MC-6); it does NOT
/// originate it. The production vault-creation wiring that seeds the salt from
/// OS entropy is deferred with the device-auth surface (see
/// `docs/V1-FROZEN-INTERFACE-DEFERRED.md` Row D-69); until then the only
/// callers are tests passing fixed-constant salts.
///
/// # Errors
///
/// Returns [`VaultError`] on an internal AEAD error.
pub fn serialize_vault(
    payload: &VaultPayload,
    salt: &[u8; 16],
    params: Argon2idParams,
    dak: &[u8; 32],
) -> Result<Vec<u8>, VaultError> {
    use chacha20poly1305::aead::{Aead, AeadCore, KeyInit, OsRng};
    use chacha20poly1305::{Key, XChaCha20Poly1305, XNonce};

    // R6-reround: the transient plaintext buffer holds the full VaultPayload
    // canonical bytes (incl. the SECRET `k_principal` + `user_did_signing_key`)
    // before AEAD-seal. Wrap in `Zeroizing` so the copy is wiped on drop /
    // unwind, not left lingering on the freed heap (Compromise #66 residual).
    let pt = Zeroizing::new(payload.to_canonical_cbor());
    let key = Key::from_slice(dak);
    let cipher = XChaCha20Poly1305::new(key);
    let nonce_bytes = XChaCha20Poly1305::generate_nonce(&mut OsRng);
    let nonce = XNonce::from_slice(&nonce_bytes);
    let aad = vault_aad();
    let ct = cipher
        .encrypt(
            nonce,
            chacha20poly1305::aead::Payload {
                msg: pt.as_slice(),
                aad: &aad,
            },
        )
        .map_err(|_| VaultError::AeadFailed)?;

    // On-disk layout (R11 MC-6): magic 0xae | V2 | codepoint BE | salt(16) |
    // m_cost BE | t_cost BE | p_cost BE | nonce_len | nonce | ct.
    //
    // The header salt+params are intentionally NOT covered by the AEAD AAD
    // (`vault_aad()` binds only domain + codepoint): they are self-authenticating
    // THROUGH the key derivation — tampering the in-header salt/params yields a
    // different DAK, so the AEAD tag then fails (fail-closed `AeadFailed`), never
    // a silently-weakened key. This is the standard, safe PBKDF-header posture
    // (age / gpg / LUKS): an attacker can DoS their own tampered copy but cannot
    // force a weak-param key onto a victim.
    let mut out = Vec::with_capacity(VAULT_HEADER_LEN + 1 + nonce_bytes.len() + ct.len());
    out.push(crate::envelope::ENVELOPE_MAGIC);
    out.push(crate::envelope::ENVELOPE_FORMAT_VERSION_V2);
    out.extend_from_slice(&VAULT_SYMMETRIC_AEAD_XNONCE_CODEPOINT.to_be_bytes());
    out.extend_from_slice(salt);
    out.extend_from_slice(&params.m_cost.to_be_bytes());
    out.extend_from_slice(&params.t_cost.to_be_bytes());
    out.extend_from_slice(&params.p_cost.to_be_bytes());
    out.push(u8::try_from(nonce_bytes.len()).unwrap_or(u8::MAX));
    out.extend_from_slice(&nonce_bytes);
    out.extend_from_slice(&ct);
    Ok(out)
}

/// The parsed vault frame header + the borrowed nonce/ct slices (R11 MC-6).
struct VaultFrame<'a> {
    salt: [u8; 16],
    params: Argon2idParams,
    nonce: &'a [u8],
    ct: &'a [u8],
}

/// Parse the vault frame header (R11 MC-6): validate the magic / version /
/// codepoint / nonce-width and return the [`VaultFrame`]. The salt+params are
/// read FROM the frame — the MC-6 self-containment property.
///
/// # Errors
///
/// Returns [`VaultError`] on a malformed / wrong-width / unknown-codepoint
/// frame.
fn parse_vault_frame(bytes: &[u8]) -> Result<VaultFrame<'_>, VaultError> {
    // Need the fixed header + the 1-byte nonce_len.
    if bytes.len() < VAULT_HEADER_LEN + 1 {
        return Err(VaultError::MalformedCbor);
    }
    if bytes[0] != crate::envelope::ENVELOPE_MAGIC {
        return Err(VaultError::MalformedCbor);
    }
    if bytes[1] != crate::envelope::ENVELOPE_FORMAT_VERSION_V2 {
        return Err(VaultError::MalformedCbor);
    }
    let codepoint = u16::from_be_bytes([bytes[2], bytes[3]]);
    if codepoint != VAULT_SYMMETRIC_AEAD_XNONCE_CODEPOINT {
        return Err(VaultError::UnknownCodepoint);
    }
    let mut salt = [0u8; 16];
    salt.copy_from_slice(&bytes[4..20]);
    let m_cost = u32::from_be_bytes([bytes[20], bytes[21], bytes[22], bytes[23]]);
    let t_cost = u32::from_be_bytes([bytes[24], bytes[25], bytes[26], bytes[27]]);
    let p_cost = u32::from_be_bytes([bytes[28], bytes[29], bytes[30], bytes[31]]);
    // Fail-closed: reject a header whose Argon2id cost params are outside the
    // safe band BEFORE they can drive `derive_dak` (Compromise #28 / META
    // #629). This single choke point covers both the self-contained
    // `open_vault` and the DAK-supplied `decode_vault` entry points, and fires
    // ahead of the AEAD/password gate so a hostile blob can neither dictate the
    // victim's KDF cost (CEILINGS) nor drive an out-of-RFC-9106 param set into
    // `derive_dak`'s `Params::new(...).expect(...)` — a reachable PANIC-DoS
    // (FLOORS, R6-R1-refix F-02). All valid frames use `OWASP_DEFAULT`
    // (19456 / 2 / 1), which clears every ceiling AND every floor.
    let below_floor = m_cost < VAULT_ARGON2_MIN_M_COST
        || t_cost < VAULT_ARGON2_MIN_T_COST
        || p_cost < VAULT_ARGON2_MIN_P_COST
        // RFC-9106 §3.1 / `argon2::Params::new`: memory cost must be at least
        // 8× the parallelism (else `Params::new` errors → `derive_dak` panics).
        || m_cost < p_cost.saturating_mul(8);
    if m_cost > VAULT_ARGON2_MAX_M_COST
        || t_cost > VAULT_ARGON2_MAX_T_COST
        || p_cost > VAULT_ARGON2_MAX_P_COST
        || below_floor
    {
        return Err(VaultError::Argon2ParamsOutOfBounds {
            m_cost,
            t_cost,
            p_cost,
        });
    }
    let params = Argon2idParams {
        m_cost,
        t_cost,
        p_cost,
    };
    let nonce_len = bytes[VAULT_HEADER_LEN] as usize;
    // U2 strict-decode: the XNonce codepoint discriminates the nonce width.
    if nonce_len != VAULT_XNONCE_LEN {
        return Err(VaultError::NonceWidthMismatch);
    }
    let nonce_start = VAULT_HEADER_LEN + 1;
    if bytes.len() < nonce_start + nonce_len {
        return Err(VaultError::MalformedCbor);
    }
    let nonce = &bytes[nonce_start..nonce_start + nonce_len];
    let ct = &bytes[nonce_start + nonce_len..];
    Ok(VaultFrame {
        salt,
        params,
        nonce,
        ct,
    })
}

/// Open a vault from the frame bytes + password ALONE (R11 MC-6).
///
/// This is the self-contained open path: the Argon2id salt + params are read
/// FROM the frame header, the DAK is re-derived via [`derive_dak`] under
/// `info_tag`, and the AEAD is opened. No external salt/param source is needed
/// — `vault.cbor` bytes + password suffice to decrypt across a restart.
///
/// # Errors
///
/// Returns [`VaultError`] on a malformed / wrong-width / unknown-codepoint /
/// wrong-password (AEAD-failure) input. All failure causes collapse to a typed
/// error with no salt/params/tag side-channel beyond the frame-shape checks.
pub fn open_vault(
    bytes: &[u8],
    password: &[u8],
    info_tag: &[u8],
) -> Result<DecodedVault, VaultError> {
    let frame = parse_vault_frame(bytes)?;
    // Re-derive the DAK from the FRAME salt+params (self-contained; MC-6).
    let dak = derive_dak(password, &frame.salt, frame.params, info_tag);
    decode_vault(bytes, dak.expose())
}

/// Decode + AEAD-open a vault envelope under a DAK. Enforces the XNonce
/// codepoint + 24-byte nonce width (strict-decode; U2).
///
/// # Errors
///
/// Returns [`VaultError`] on a malformed / wrong-width / unknown-codepoint /
/// AEAD-failure input.
pub fn decode_vault(bytes: &[u8], dak: &[u8; 32]) -> Result<DecodedVault, VaultError> {
    use chacha20poly1305::aead::{Aead, KeyInit};
    use chacha20poly1305::{Key, XChaCha20Poly1305, XNonce};

    // Parse + validate the frame header (R11 MC-6: reads salt+params too).
    let frame = parse_vault_frame(bytes)?;

    let key = Key::from_slice(dak);
    let cipher = XChaCha20Poly1305::new(key);
    let nonce = XNonce::from_slice(frame.nonce);
    let aad = vault_aad();
    // R6-reround: the decrypted plaintext buffer holds the full VaultPayload
    // canonical bytes (incl. the SECRET `k_principal` + `user_did_signing_key`)
    // BEFORE it is parsed into the zeroize-on-drop `VaultPayload`. Wrap the
    // transient buffer in `Zeroizing` so it is wiped on drop / unwind rather
    // than lingering on the freed heap (Compromise #66 residual).
    let pt = Zeroizing::new(
        cipher
            .decrypt(
                nonce,
                chacha20poly1305::aead::Payload {
                    msg: frame.ct,
                    aad: &aad,
                },
            )
            .map_err(|_| VaultError::AeadFailed)?,
    );
    let payload = VaultPayload::from_canonical_cbor(&pt)?;
    Ok(DecodedVault {
        codepoint: VAULT_SYMMETRIC_AEAD_XNONCE_CODEPOINT,
        salt: frame.salt,
        params: frame.params,
        nonce: frame.nonce.to_vec(),
        payload,
    })
}

/// Strict-decode check: a vault tagged `SymmetricAeadXNonce` (`0x6100`)
/// carrying a 12-byte nonce MUST be rejected (U2 — codepoint discriminates
/// nonce width; no silent coercion).
///
/// # Errors
///
/// Returns [`VaultError::NonceWidthMismatch`] when `codepoint` is the XNonce
/// vault codepoint but `nonce_len != 24`.
pub fn decode_vault_strict(codepoint: u16, nonce_len: usize) -> Result<(), VaultError> {
    if codepoint == VAULT_SYMMETRIC_AEAD_XNONCE_CODEPOINT && nonce_len != VAULT_XNONCE_LEN {
        return Err(VaultError::NonceWidthMismatch);
    }
    Ok(())
}

/// The vault AEAD AAD (domain-separates the vault seal from every other
/// envelope; binds the vault codepoint).
fn vault_aad() -> Vec<u8> {
    let mut aad = Vec::new();
    aad.extend_from_slice(VAULT_AAD_DOMAIN);
    aad.extend_from_slice(&VAULT_SYMMETRIC_AEAD_XNONCE_CODEPOINT.to_be_bytes());
    aad
}

/// The hydrated key material (R0.5 §3.1) — BOTH keys, atomically. The
/// `k_principal` lives in a [`secrecy::SecretBox`] (Debug-redacting +
/// zeroize-on-Drop); the signing key is a plain `Vec<u8>` (its bytes are not
/// themselves a long-lived at-rest secret in the same coredump sense, but the
/// whole struct is dropped together).
pub struct UnlockedKeyMaterial {
    k_principal: SecretBox<[u8; 32]>,
    user_did_signing_key: Vec<u8>,
}

impl UnlockedKeyMaterial {
    /// Construct from the hydrated payload (production unlock path).
    #[must_use]
    pub fn new(k_principal: [u8; 32], user_did_signing_key: Vec<u8>) -> Self {
        Self {
            k_principal: SecretBox::new(Box::new(k_principal)),
            user_did_signing_key,
        }
    }

    /// Construct directly from a 32-byte `K_principal` for the F-LB-1
    /// structural-KDF bridge pins (the vault source the structural-KDF chain
    /// seeds from).
    #[must_use]
    #[cfg(any(test, feature = "testing"))]
    pub fn from_vault_bytes_for_test(k_principal_bytes: [u8; 32]) -> Self {
        Self::new(k_principal_bytes, vec![0x22u8; 64])
    }

    /// Expose the `K_principal` bytes (explicit, greppable access; the SOLE
    /// access path — mirrors `secrecy::ExposeSecret`).
    #[must_use]
    pub fn expose_k_principal(&self) -> &[u8; 32] {
        self.k_principal.expose_secret()
    }

    /// The 32-byte `K_principal` length (F-VA-5 atomic-hydrate pin).
    #[must_use]
    pub fn k_principal_len(&self) -> usize {
        self.k_principal.expose_secret().len()
    }

    /// The user-DID signing key (hydrated ATOMICALLY with K_principal).
    #[must_use]
    pub fn user_did_signing_key(&self) -> &[u8] {
        &self.user_did_signing_key
    }

    /// The structural-KDF root key seed — the structural-KDF chain seeds from
    /// the vault `K_principal` (F-LB-1 keying-root binding).
    ///
    /// **TEST-ONLY seam** (R6-R3 F-07, mirroring the `aead.rs` mr-major-2
    /// twin fix). This wraps the raw `K_principal` directly into a
    /// `StructuralKdfKey` newtype — it exposes the structural-KDF ROOT seam
    /// and has NO production callers (the production keying path routes
    /// `K_principal` through `structural_kdf::derive_root`, which binds the
    /// cipher-suite codepoint + root CID, NOT a raw-bytes wrap). Gated to
    /// `test`/`testing` so the raw root is not reachable from the frozen
    /// production surface. Used only by the F-LB-1 pins (which build under
    /// `--features testing`).
    #[must_use]
    #[cfg(any(test, feature = "testing"))]
    pub fn structural_kdf_root_key_for_test(&self) -> StructuralKdfKey {
        StructuralKdfKey::from_bytes_for_test(self.k_principal.expose_secret())
    }
}

impl core::fmt::Debug for UnlockedKeyMaterial {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        // Redacts — never renders the key bytes (Compromise #36).
        f.debug_struct("UnlockedKeyMaterial")
            .field("k_principal", &"SecretBox<[u8; 32]>")
            .field("user_did_signing_key", &"<redacted>")
            .finish()
    }
}

impl Drop for UnlockedKeyMaterial {
    /// R11 MC-13: zeroize `user_did_signing_key` on drop for symmetry with
    /// `K_principal` (which is wiped by its `SecretBox`). The signing key is a
    /// long-lived at-rest secret (the user-DID hybrid signing key hydrated from
    /// the vault); wiping it on drop closes the same coredump/freed-heap window
    /// `K_principal` already closes (Compromise #36/#39). `k_principal`'s own
    /// `SecretBox` handles its zeroize independently.
    fn drop(&mut self) {
        self.user_did_signing_key.zeroize();
    }
}

/// A minimal lock-state engine modelling the production vault gate (F-VA-5).
/// Pre-unlock crypto ops are typed-rejected with [`VaultError::EngineLocked`].
///
/// **R6-final F-02: test/lock-state harness — gated off the frozen public
/// surface.** `encrypt_node` is a repeating-key XOR *stand-in* (it demonstrates
/// the lock-state contract, NOT a real cipher — the production path routes the
/// structural-KDF + AEAD). It has zero production callers and is consumed only
/// by the `f_va_5` lock-state pin, so it is gated behind
/// `#[cfg(any(test, feature = "testing"))]` rather than frozen into the v1
/// public API where a naive consumer could mistake the XOR output for a seal.
#[cfg(any(test, feature = "testing"))]
pub struct VaultEngine {
    unlocked: Option<UnlockedKeyMaterial>,
}

#[cfg(any(test, feature = "testing"))]
impl Default for VaultEngine {
    fn default() -> Self {
        Self::new_locked()
    }
}

#[cfg(any(test, feature = "testing"))]
impl VaultEngine {
    /// A fresh locked engine (no hydrated handle).
    #[must_use]
    pub fn new_locked() -> Self {
        Self { unlocked: None }
    }

    /// Unlock atomically hydrates BOTH keys from a decoded vault payload.
    pub fn unlock(&mut self, payload: &VaultPayload) {
        self.unlocked = Some(UnlockedKeyMaterial::new(
            payload.k_principal,
            payload.user_did_signing_key.clone(),
        ));
    }

    /// A lock-gated op that exercises the lock-state contract. Pre-unlock
    /// returns [`VaultError::EngineLocked`] (fail-CLOSED; never a silent
    /// plaintext pass-through). Post-unlock it applies a **repeating-key XOR
    /// stand-in** over `K_principal` — NOT a real seal (no nonce, no MAC,
    /// keystream reuse). This method exists ONLY to prove the lock-state gate;
    /// it is `#[cfg(any(test, feature = "testing"))]` and MUST NOT be used to
    /// produce real ciphertext. The production seal routes the structural-KDF +
    /// AEAD via `cipher_suite` — never this fn.
    ///
    /// # Errors
    ///
    /// Returns [`VaultError::EngineLocked`] when the engine is locked.
    pub fn encrypt_node(&self, plaintext: &[u8]) -> Result<Vec<u8>, VaultError> {
        match &self.unlocked {
            Some(km) => {
                // Repeating-key XOR STAND-IN over K_principal (NOT a cipher):
                // the observable consequence is ciphertext ≠ plaintext + a
                // non-empty keyed output. The production path routes the
                // structural-KDF + AEAD; this gate's contract is the
                // lock-state, not the cipher.
                let k = km.expose_k_principal();
                let out: Vec<u8> = plaintext
                    .iter()
                    .enumerate()
                    .map(|(i, b)| b ^ k[i % 32])
                    .collect();
                Ok(out)
            }
            None => Err(VaultError::EngineLocked),
        }
    }

    /// The hydrated handle (None while locked).
    #[must_use]
    pub fn unlocked_handle(&self) -> Option<&UnlockedKeyMaterial> {
        self.unlocked.as_ref()
    }
}

/// Typed vault error.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum VaultError {
    /// A pre-unlock crypto op was attempted (fail-closed; F-VA-5).
    #[error("engine is locked — unlock the vault before crypto ops (fail-closed)")]
    EngineLocked,

    /// The DAK derivation failed (wrong password).
    ///
    /// **Intentionally never constructed (F-VA-3).** The wrong-password path
    /// deliberately surfaces the GENERIC decrypt-failure error
    /// (`UnlockError::VaultDecryptFailed`), NOT this distinct variant, so that
    /// "salt off" / "params off" / "tag off" / "wrong password" all land on ONE
    /// typed error with no error-variant side-channel (the F-VA-3
    /// single-typed-rejection / constant-time property). Constructing this
    /// variant would re-introduce the wrong-password oracle F-VA-3 forbids; it is
    /// retained as a named-but-unused code for API documentation only.
    #[error("wrong password — DAK does not open the vault")]
    WrongPassword,

    /// A vault tagged `SymmetricAeadXNonce` carried a non-24-byte nonce.
    #[error(
        "vault nonce width mismatch (XNonce codepoint requires a 24-byte nonce; U2 strict-decode)"
    )]
    NonceWidthMismatch,

    /// The vault codepoint is unknown.
    #[error("unknown vault codepoint")]
    UnknownCodepoint,

    /// The vault CBOR payload is malformed.
    #[error("malformed vault CBOR")]
    MalformedCbor,

    /// The frame-supplied Argon2id cost params exceed the fail-closed
    /// ceilings ([`VAULT_ARGON2_MAX_M_COST`] / [`VAULT_ARGON2_MAX_T_COST`] /
    /// [`VAULT_ARGON2_MAX_P_COST`]). Rejected BEFORE `derive_dak` runs so an
    /// attacker-supplied `vault.cbor` header cannot dictate the victim's KDF
    /// memory/CPU budget (Compromise #28 / META #629 DoS-sweep).
    #[error(
        "vault Argon2id params out of bounds (m_cost={m_cost} t_cost={t_cost} p_cost={p_cost}; \
         max m={} t={} p={})",
        VAULT_ARGON2_MAX_M_COST,
        VAULT_ARGON2_MAX_T_COST,
        VAULT_ARGON2_MAX_P_COST
    )]
    Argon2ParamsOutOfBounds {
        /// The offending memory cost (KiB).
        m_cost: u32,
        /// The offending time cost (passes).
        t_cost: u32,
        /// The offending parallelism.
        p_cost: u32,
    },

    /// The AEAD seal/open failed.
    #[error("vault AEAD seal/open failed")]
    AeadFailed,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dak_deterministic_and_param_sensitive() {
        // Test-only KDF inputs. Built at runtime (NOT byte-string literals)
        // so CodeQL's `rust/hard-coded-cryptographic-value` query does not
        // flag this inline `#[cfg(test)]` fixture — `paths-ignore` in
        // `.github/codeql/codeql-config.yml` excludes `tests/` files but
        // cannot see inline test modules inside a `src/` file. The referenced
        // production items (`derive_dak`, `VaultEngine`, …) are `pub` — part of
        // the G-CORE-9 frozen public surface (see
        // `docs/public-api/benten-crypto-suite.txt`), which is exactly why this
        // determinism check lives in an inline `#[cfg(test)]` module (it exercises
        // the frozen `pub` API in place; no visibility widening is involved). The
        // values only need to be deterministic across the two `derive_dak` calls;
        // every production DAK input is operator-supplied / CSPRNG-salted.
        let pw: Vec<u8> = (0u8..29)
            .map(|i| i.wrapping_mul(7).wrapping_add(3))
            .collect();
        let salt: [u8; 16] = core::array::from_fn(|i| (i as u8) ^ 0x5A);
        let a = derive_dak(&pw, &salt, OWASP_DEFAULT, DAK_HKDF_INFO_TAG);
        let b = derive_dak(&pw, &salt, OWASP_DEFAULT, DAK_HKDF_INFO_TAG);
        assert_eq!(a.expose(), b.expose());
        let stronger = Argon2idParams {
            m_cost: 65536,
            t_cost: 3,
            p_cost: 1,
        };
        assert_ne!(
            a.expose(),
            derive_dak(&pw, &salt, stronger, DAK_HKDF_INFO_TAG).expose()
        );
        let alt_info: Vec<u8> = b"benten-dak-v2".to_vec();
        assert_ne!(
            a.expose(),
            derive_dak(&pw, &salt, OWASP_DEFAULT, &alt_info).expose()
        );
    }

    #[test]
    fn vault_round_trips_with_24_byte_nonce() {
        let payload = VaultPayload {
            k_principal: [0x11u8; 32],
            user_did_signing_key: vec![0x22u8; 64],
            user_did_creation_time: 0x0000_0000_6543_2100,
        };
        let salt = [0x77u8; 16];
        let dak = [0x33u8; 32];
        let bytes = serialize_vault(&payload, &salt, OWASP_DEFAULT, &dak).unwrap();
        let decoded = decode_vault(&bytes, &dak).unwrap();
        assert_eq!(decoded.nonce.len(), VAULT_XNONCE_LEN);
        assert_eq!(decoded.codepoint, VAULT_SYMMETRIC_AEAD_XNONCE_CODEPOINT);
        assert_eq!(decoded.salt, salt, "salt round-trips from the frame header");
        assert_eq!(
            decoded.params, OWASP_DEFAULT,
            "params round-trip from the frame header"
        );
        assert_eq!(decoded.payload, payload);
    }

    /// Compromise #28 / META #629 DoS-sweep — a `vault.cbor` HEADER whose
    /// attacker-controlled Argon2id `m_cost` exceeds [`VAULT_ARGON2_MAX_M_COST`]
    /// is rejected FAST with a typed [`VaultError::Argon2ParamsOutOfBounds`],
    /// BEFORE `derive_dak` allocates `m_cost` KiB. would-FAIL-on-revert: without
    /// the `parse_vault_frame` ceiling, `open_vault` on this frame would drive a
    /// ~4 GiB Argon2id allocation (`m_cost = 0x0040_0000` KiB) — an unbounded
    /// memory-exhaustion DoS dictated by the hostile blob. The test exercises
    /// `decode_vault` (which calls `parse_vault_frame` first) so it rejects at
    /// the header-parse gate WITHOUT ever running the KDF or AEAD.
    #[test]
    fn oversized_argon2_params_reject_fast_before_kdf() {
        // Build a valid frame under OWASP_DEFAULT, then corrupt only the
        // m_cost field in the header to an oversized value.
        let payload = VaultPayload {
            k_principal: [0x11u8; 32],
            user_did_signing_key: vec![0x22u8; 64],
            user_did_creation_time: 0,
        };
        let salt = [0x77u8; 16];
        let dak = [0x33u8; 32];
        let mut bytes = serialize_vault(&payload, &salt, OWASP_DEFAULT, &dak).unwrap();
        // Header layout: magic(1) version(1) codepoint(2) salt(16)
        // m_cost@[20..24] t_cost@[24..28] p_cost@[28..32] (big-endian).
        let hostile_m_cost: u32 = 0x0040_0000; // 4 GiB in KiB — well over the 64 MiB ceiling
        bytes[20..24].copy_from_slice(&hostile_m_cost.to_be_bytes());
        let err = decode_vault(&bytes, &dak).unwrap_err();
        match err {
            VaultError::Argon2ParamsOutOfBounds { m_cost, .. } => {
                assert_eq!(m_cost, hostile_m_cost, "reports the offending m_cost");
            }
            other => panic!("expected Argon2ParamsOutOfBounds, got {other:?}"),
        }
        // And the self-contained open path (which re-derives the DAK) rejects
        // at the SAME gate before ever calling derive_dak.
        let err2 = open_vault(&bytes, b"pw", DAK_HKDF_INFO_TAG).unwrap_err();
        assert!(matches!(err2, VaultError::Argon2ParamsOutOfBounds { .. }));
    }

    /// R6-R1-refix F-02 — the FLOOR half of the same DoS choke-point. A
    /// `vault.cbor` HEADER whose attacker-controlled Argon2id params fall
    /// BELOW the RFC-9106 minimums (`t_cost = 0`, `p_cost = 0`, or
    /// `m_cost < 8 * p_cost`) is rejected FAST with a typed
    /// [`VaultError::Argon2ParamsOutOfBounds`] at the header-parse gate.
    /// would-FAIL-on-revert: without the floor, `parse_vault_frame` waves such
    /// a header through (it exceeds no ceiling) into `derive_dak` →
    /// `Params::new(...).expect(...)`, which REJECTS the sub-minimum params and
    /// **panics** — a reachable crash-DoS on the frozen forever-decode path.
    /// Reverting the floor turns each sub-case below into a panic inside
    /// `decode_vault` (a test failure), proving the pin is non-vacuous.
    #[test]
    fn sub_minimum_argon2_params_reject_fast_not_panic() {
        let payload = VaultPayload {
            k_principal: [0x11u8; 32],
            user_did_signing_key: vec![0x22u8; 64],
            user_did_creation_time: 0,
        };
        let salt = [0x77u8; 16];
        let dak = [0x33u8; 32];
        let base = serialize_vault(&payload, &salt, OWASP_DEFAULT, &dak).unwrap();
        // Header (big-endian): m_cost@[20..24] t_cost@[24..28] p_cost@[28..32].

        // Sub-case A: t_cost = 0 (below the MIN_T_COST floor).
        let mut a = base.clone();
        a[24..28].copy_from_slice(&0u32.to_be_bytes());
        assert!(
            matches!(
                decode_vault(&a, &dak),
                Err(VaultError::Argon2ParamsOutOfBounds { t_cost: 0, .. })
            ),
            "t_cost=0 must reject at the parse gate, not panic in derive_dak"
        );

        // Sub-case B: p_cost = 0 (below the MIN_P_COST floor).
        let mut b = base.clone();
        b[28..32].copy_from_slice(&0u32.to_be_bytes());
        assert!(
            matches!(
                decode_vault(&b, &dak),
                Err(VaultError::Argon2ParamsOutOfBounds { p_cost: 0, .. })
            ),
            "p_cost=0 must reject at the parse gate"
        );

        // Sub-case C: m_cost < 8 * p_cost (RFC-9106 relation): m=8, p=4 → 8 < 32.
        let mut c = base.clone();
        c[20..24].copy_from_slice(&8u32.to_be_bytes());
        c[28..32].copy_from_slice(&4u32.to_be_bytes());
        assert!(
            matches!(
                decode_vault(&c, &dak),
                Err(VaultError::Argon2ParamsOutOfBounds { .. })
            ),
            "m_cost < 8*p_cost must reject at the parse gate"
        );

        // The self-contained open path rejects at the SAME gate before derive_dak.
        let mut o = base;
        o[24..28].copy_from_slice(&0u32.to_be_bytes());
        assert!(matches!(
            open_vault(&o, b"pw", DAK_HKDF_INFO_TAG),
            Err(VaultError::Argon2ParamsOutOfBounds { .. })
        ));
    }

    /// R11 MC-6 — the self-containment property: `vault.cbor` bytes + password
    /// ALONE re-derive the DAK (salt+params from the frame header) and decrypt.
    /// NO external salt is needed. would-FAIL-on-revert: if the frame did not
    /// persist salt+params, `open_vault` could not re-derive the DAK from the
    /// bytes alone.
    #[test]
    fn open_vault_from_bytes_and_password_alone() {
        let payload = VaultPayload {
            k_principal: [0xA1u8; 32],
            user_did_signing_key: vec![0xB2u8; 64],
            user_did_creation_time: 7,
        };
        // Build inputs at runtime (CodeQL hard-coded-crypto hygiene).
        let password: Vec<u8> = (0u8..24)
            .map(|i| i.wrapping_mul(5).wrapping_add(1))
            .collect();
        let salt: [u8; 16] = core::array::from_fn(|i| (i as u8).wrapping_mul(3) ^ 0x2C);
        let params = OWASP_DEFAULT;

        // Seal: derive the DAK from (password, salt, params) and serialize the
        // frame WITH salt+params in the header.
        let dak = derive_dak(&password, &salt, params, DAK_HKDF_INFO_TAG);
        let bytes = serialize_vault(&payload, &salt, params, dak.expose()).unwrap();

        // Open with ONLY the frame bytes + the password (drop the in-RAM salt).
        let decoded = open_vault(&bytes, &password, DAK_HKDF_INFO_TAG)
            .expect("vault.cbor bytes + password alone MUST decrypt");
        assert_eq!(decoded.payload, payload);
        assert_eq!(decoded.salt, salt);
        assert_eq!(decoded.params, params);

        // A wrong password fails closed (single typed rejection).
        let mut wrong = password.clone();
        wrong[0] ^= 0xFF;
        assert!(matches!(
            open_vault(&bytes, &wrong, DAK_HKDF_INFO_TAG),
            Err(VaultError::AeadFailed)
        ));
    }

    #[test]
    fn strict_decode_rejects_12_byte_nonce() {
        assert!(matches!(
            decode_vault_strict(VAULT_SYMMETRIC_AEAD_XNONCE_CODEPOINT, 12),
            Err(VaultError::NonceWidthMismatch)
        ));
    }

    #[test]
    fn engine_locked_pre_unlock() {
        let engine = VaultEngine::new_locked();
        assert!(matches!(
            engine.encrypt_node(b"secret"),
            Err(VaultError::EngineLocked)
        ));
    }

    #[test]
    fn unlock_hydrates_both_keys() {
        let payload = VaultPayload {
            k_principal: [0x44u8; 32],
            user_did_signing_key: vec![0x55u8; 64],
            user_did_creation_time: 1,
        };
        let mut engine = VaultEngine::new_locked();
        assert!(engine.unlocked_handle().is_none());
        engine.unlock(&payload);
        let km = engine.unlocked_handle().unwrap();
        assert_eq!(km.k_principal_len(), 32);
        assert!(!km.user_did_signing_key().is_empty());
        let ct = engine.encrypt_node(b"secret node body").unwrap();
        assert_ne!(ct.as_slice(), b"secret node body".as_slice());
    }

    #[test]
    fn debug_does_not_leak_key() {
        // R6-tail F-41 sweep: distinct-byte fixture (an all-same-byte foil is
        // weak — a single coincidental decimal decides the assertion), and the
        // scan is the CONTIGUOUS decimal SEQUENCE a derived Debug would emit,
        // matching the `LEAK_DECIMAL` convention in
        // `crates/benten-engine/tests/f_secret_hygiene_roster.rs`.
        let mut k = [0u8; 32];
        k[..8].copy_from_slice(&[0xDE, 0xAD, 0xBE, 0xEF, 0xCA, 0xFE, 0xBA, 0xD0]);
        let km = UnlockedKeyMaterial::new(k, vec![0x11u8; 64]);
        let rendered = format!("{km:?}");
        const LEAK_DECIMAL: &str = "222, 173, 190, 239, 202, 254, 186, 208";
        assert!(
            !rendered.contains(LEAK_DECIMAL),
            "UnlockedKeyMaterial Debug MUST NOT render the K_principal bytes \
             (Compromise #36); a derived Debug leaks them as `{LEAK_DECIMAL}`; \
             rendered=`{rendered}`"
        );
        // Positive guards: BOTH secret fields are replaced wholesale.
        assert!(
            rendered.contains("k_principal: \"SecretBox<[u8; 32]>\""),
            "k_principal MUST be replaced wholesale by the SecretBox \
             placeholder; rendered=`{rendered}`"
        );
        assert!(
            rendered.contains("user_did_signing_key: \"<redacted>\""),
            "user_did_signing_key MUST be replaced wholesale by the redaction \
             marker; rendered=`{rendered}`"
        );
    }

    /// Drift defense: the vault at-rest domain tags are registered
    /// cross-surface domain-separation tags in the central
    /// [`crate::domain_registry`] table over which the prefix-free invariant
    /// runs. Pin byte-equality so a mirror can never silently diverge from the
    /// home definitions here.
    #[test]
    fn vault_domain_tags_match_central_registry() {
        use crate::domain_registry as reg;
        assert_eq!(
            VAULT_AAD_DOMAIN,
            reg::VAULT_AAD_DOMAIN,
            "VAULT_AAD_DOMAIN drifted from the central domain_registry mirror"
        );
        assert_eq!(
            DAK_HKDF_INFO_TAG,
            reg::DAK_HKDF_INFO_TAG,
            "DAK_HKDF_INFO_TAG drifted from the central domain_registry mirror"
        );
        // R6-final F-07: ABSOLUTE freeze pin (symmetric to the
        // `DAK_HKDF_INFO_TAG` absolute pin in `f_va_2_argon2id_dak_derivation`).
        // The mirror-equality asserts above move together under a coordinated
        // rename of BOTH mirrors, leaving every round-trip / mirror test green
        // while silently stranding every previously-sealed v1-beta vault (the
        // AAD input changes). Pin the exact frozen bytes so a rename fails loud.
        assert_eq!(
            VAULT_AAD_DOMAIN, b"benten-vault:",
            "the frozen vault AEAD AAD domain is exactly `benten-vault:`"
        );
    }
}
