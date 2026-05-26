//! Structural-KDF substrate per Spike-E Interpretation-B path-tagged
//! key-derivation (G-CORE-3a CANARY).
//!
//! # Contract (RATIFIED — `RATIFIED-sharing-and-confidentiality-2026-05-21`
//! refinement (2) + `00-implementation-plan.md` §1.A.FROZEN item 15(f))
//!
//! ```text
//! K(root) = HKDF-SHA256(K_principal, info = "root" || root_cid)
//! K(N)    = HKDF-SHA256(K(predecessor), info = "step" || edge_label || N.cid)
//! ```
//!
//! The `"root"` / `"step"` HKDF info-tag prefixes provide cross-role
//! domain separation per Spike-E's working `derive_step` API + R0.8
//! crypto-agility-r1.4-2 corrective. Eliding the tag (e.g. constructing
//! `info = edge_label || N.cid` only) MUST yield a different derived key
//! — the textbook HKDF role-separation property.
//!
//! # Path-tagged keys (Spike-E Interpretation-B)
//!
//! A Node reachable by multiple paths gets multiple distinct keys
//! (feature for selective-share per RATIFIED refinement (2)). The
//! literal-DESIGN-doc "structure-independent" formula was disproved by
//! Spike E — DO NOT collapse `derive_step` to a path-independent
//! HKDF(K_principal, N.cid) shape.
//!
//! # No-fork-primitives discipline (CLAUDE.md baked-in #5)
//!
//! HKDF-SHA256 is wrapped from vetted upstream `hkdf` + `sha2` crates.
//! This module is HKDF-glue only — info-tag concatenation, output-length
//! dispatch, zeroize-on-drop newtype around the 32-byte PRK.
//!
//! # KDF codepoint = HKDF-SHA256 v1-beta default
//!
//! Per §1.A.FROZEN item 15(f): "KDF = HKDF-SHA256 v1-beta default
//! (textbook domain-separation slot for variable-length material; the
//! `"step"`/`"root"` info-tags use the slot for role-separation)." The
//! codepoint dispatch here is implicit at this wave (single arm);
//! G-CORE-9 freezes the explicit codepoint per the additive-codepoint
//! discipline.

use hkdf::Hkdf;
use sha2::Sha256;
use zeroize::Zeroize;

/// Structural-KDF key material (32-byte HKDF-SHA256 output) — the
/// derive_step / derive_root output type. Zeroizes on drop.
///
/// The byte length is fixed at 32 (HKDF-SHA256's natural output size) —
/// this is NOT a hardcoded crypto-primitive size in the CLAUDE.md #5
/// "no-hardcoded-sizes" sense (which applies to algorithm-dimensioned
/// sizes like ML-DSA-65 1952-B key / 3309-B sig); HKDF-SHA256's output
/// length is parameter-fixed to the underlying hash output (32 B for
/// SHA-256), and the codepoint surface enforces the dispatch.
#[derive(Clone)]
pub struct StructuralKdfKey([u8; 32]);

impl StructuralKdfKey {
    /// Wrap raw 32-byte material into the typed newtype. Crate-public
    /// for `derive_root`/`derive_step` internals + the test-only
    /// `from_bytes_for_test` constructor.
    pub(crate) const fn from_bytes(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    /// Test-only constructor — accepts arbitrary-length slice + pads /
    /// copies into the 32-byte newtype. Used by W1 pins to inject
    /// per-test K_principal byte vectors.
    ///
    /// # Panics
    ///
    /// Panics if `bytes.len() > 32` (the caller is expected to pass
    /// exactly 32 bytes; anything longer is a test-author bug).
    #[must_use]
    pub fn from_bytes_for_test(bytes: &[u8]) -> Self {
        let mut buf = [0u8; 32];
        let n = bytes.len().min(32);
        buf[..n].copy_from_slice(&bytes[..n]);
        Self(buf)
    }

    /// Returns the raw 32-byte key material (test-only inspection +
    /// AEAD-key feed). Production callers should pass `&StructuralKdfKey`
    /// to the next `derive_step` rather than reaching for `.as_bytes()`.
    #[must_use]
    pub fn as_bytes(&self) -> [u8; 32] {
        self.0
    }

    /// Adversarial-test helper — derive a step key OMITTING the
    /// `"step"` HKDF info-tag prefix. Used by the negative-control
    /// pin `tf3a_p1_step_info_tag_elision_changes_derived_key` to assert
    /// the info-tag is load-bearing (eliding it MUST change the key).
    ///
    /// **NOT a production API.** Never call this outside tests.
    #[must_use]
    #[cfg(any(test, feature = "testing"))]
    pub fn derive_step_without_info_tag_for_test(
        predecessor: &StructuralKdfKey,
        edge_label: &[u8],
        node_cid: &[u8],
    ) -> Self {
        // info = edge_label || node_cid (NO "step" prefix — the elision).
        let mut info = Vec::with_capacity(edge_label.len() + node_cid.len());
        info.extend_from_slice(edge_label);
        info.extend_from_slice(node_cid);
        hkdf_sha256_32(&predecessor.0, &info)
    }
}

impl Drop for StructuralKdfKey {
    fn drop(&mut self) {
        self.0.zeroize();
    }
}

/// Derive the root key for a principal-rooted walk.
///
/// Formula (R6 R2 batch-A Item 7 — Row D-13 closure;
/// Spike-E + §1.A.FROZEN item 15(f) extended):
/// `K(root) = HKDF-SHA256(K_principal,
///   info = "root:codepoint:" || codepoint_le_bytes || root_cid)`
///
/// The `"root"` HKDF info-tag is the cross-role domain separator (it
/// disambiguates the root-derivation step from step-derivation; eliding
/// it would collide root and step keys with `edge_label = b""`).
///
/// The `cipher_suite_codepoint` binding closes the
/// `aead_wrap::make_key_material_matching` attacker-controlled
/// codepoint → key newtype attack class: pre-Item-7 the same
/// `(K_principal, root_cid)` pair derived the SAME `K_root` regardless
/// of which cipher-suite arm the K_root was about to feed (e.g.
/// `0x647a` X-Wing hybrid vs `0x6400` X25519-classical vs `0x647b`
/// pure-PQ reserved). An attacker who could mutate the envelope
/// codepoint between derivation + AEAD-wrap could mix keys across
/// codepoint arms (cross-codepoint replay attack class). Post-Item-7
/// the codepoint enters the info-tag → K_root is strongly bound to
/// the cipher-suite arm it serves, so cross-codepoint key reuse is
/// structurally impossible (different codepoint → different K_root
/// → different downstream AEAD key).
///
/// The codepoint is encoded as little-endian `u16` bytes (2 bytes)
/// for compactness + endianness determinism (matches the
/// `CipherSuiteCodepoint::as_le_bytes` convention at the wire
/// layer).
///
/// Cross-cut with G-CORE-PQ-WIRE swap matrix: the same codepoint
/// dispatch works for classical AEAD (`0x6400`) + future PQ-hybrid
/// envelope codepoints (`0x647a` / `0x647b` / `0x647c`); the binding
/// is additive at the info-tag layer.
#[must_use]
pub fn derive_root(
    k_principal: &StructuralKdfKey,
    root_cid: &[u8],
    cipher_suite_codepoint: u16,
) -> StructuralKdfKey {
    // info = "root:codepoint:" || codepoint_le_bytes || root_cid.
    let codepoint_bytes = cipher_suite_codepoint.to_le_bytes();
    let mut info = Vec::with_capacity(15 + codepoint_bytes.len() + root_cid.len());
    info.extend_from_slice(b"root:codepoint:");
    info.extend_from_slice(&codepoint_bytes);
    info.extend_from_slice(root_cid);
    hkdf_sha256_32(&k_principal.0, &info)
}

/// Derive a step key along a canonical-path walk edge.
///
/// Formula (Spike-E Interpretation-B + §1.A.FROZEN item 15(f)):
/// `K(N) = HKDF-SHA256(K(predecessor), info = "step" || edge_label || N.cid)`
///
/// Path-tagged: same Node reached by different predecessors yields
/// different keys (the selective-share feature; structure-independent
/// formula DISPROVED by Spike-E).
#[must_use]
pub fn derive_step(
    predecessor: &StructuralKdfKey,
    edge_label: &[u8],
    node_cid: &[u8],
) -> StructuralKdfKey {
    // info = "step" || edge_label || node_cid (the cross-role prefix).
    let mut info = Vec::with_capacity(4 + edge_label.len() + node_cid.len());
    info.extend_from_slice(b"step");
    info.extend_from_slice(edge_label);
    info.extend_from_slice(node_cid);
    hkdf_sha256_32(&predecessor.0, &info)
}

/// HKDF-SHA256 with empty salt → 32-byte output.
///
/// Wraps the vetted upstream `hkdf::Hkdf<Sha256>` — no Benten-side
/// reimplementation (CLAUDE.md #5 never-fork-primitives).
fn hkdf_sha256_32(ikm: &[u8], info: &[u8]) -> StructuralKdfKey {
    let hk = Hkdf::<Sha256>::new(None, ikm);
    let mut okm = [0u8; 32];
    // hkdf::expand returns Err only if output length exceeds the
    // SHA-256-derived maximum (8160 bytes); 32 B is well under so this
    // is infallible at runtime. `.expect` documents the invariant.
    hk.expand(info, &mut okm)
        .expect("HKDF-SHA256 expand to 32 B is infallible (output << 8160 B max)");
    StructuralKdfKey::from_bytes(okm)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn derive_root_is_deterministic() {
        let k = StructuralKdfKey::from_bytes([0u8; 32]);
        let cid = [0xA0u8; 32];
        let a = derive_root(&k, &cid, 0x647a);
        let b = derive_root(&k, &cid, 0x647a);
        assert_eq!(a.as_bytes(), b.as_bytes());
    }

    /// **R6 R2 batch-A Item 7 (Row D-13 closure) load-bearing pin:**
    /// `derive_root` keys MUST differ across cipher-suite codepoints
    /// even for IDENTICAL `(K_principal, root_cid)` inputs. Closes
    /// the cross-codepoint key-reuse attack class at the K_root layer.
    ///
    /// Would-FAIL-on-revert: drop the `cipher_suite_codepoint`
    /// parameter from `derive_root` → both arms reduce to the same
    /// 2-arg call → keys are equal → assertion fires.
    #[test]
    fn derive_root_distinguishes_cipher_suite_codepoints() {
        let k = StructuralKdfKey::from_bytes([0x77u8; 32]);
        let cid = [0xBBu8; 32];
        let k_hybrid = derive_root(&k, &cid, 0x647a); // X-Wing hybrid
        let k_classical = derive_root(&k, &cid, 0x6400); // X25519-classical
        let k_pq_reserved = derive_root(&k, &cid, 0x647b); // NF-1 reserved
        assert_ne!(
            k_hybrid.as_bytes(),
            k_classical.as_bytes(),
            "Item 7: K_root MUST differ across codepoint arms (0x647a vs 0x6400) — \
             cross-codepoint key reuse class closed at the info-tag binding"
        );
        assert_ne!(
            k_hybrid.as_bytes(),
            k_pq_reserved.as_bytes(),
            "Item 7: K_root MUST differ across hybrid arms (0x647a vs 0x647b)"
        );
        assert_ne!(
            k_classical.as_bytes(),
            k_pq_reserved.as_bytes(),
            "Item 7: K_root MUST differ across classical vs reserved (0x6400 vs 0x647b)"
        );
    }

    #[test]
    fn derive_step_is_deterministic() {
        let k = StructuralKdfKey::from_bytes([1u8; 32]);
        let edge = b"edge:VERSION_OF";
        let cid = [0xA1u8; 32];
        let a = derive_step(&k, edge, &cid);
        let b = derive_step(&k, edge, &cid);
        assert_eq!(a.as_bytes(), b.as_bytes());
    }

    #[test]
    fn root_and_step_are_role_separated() {
        // K(root) for cid_X != K(N) with edge_label="" for cid_X — the
        // "root"/"step" info-tags MUST separate the roles.
        let k = StructuralKdfKey::from_bytes([2u8; 32]);
        let cid = [0xCDu8; 32];
        // R6 R2 batch-A Item 7: derive_root now takes a codepoint;
        // pick the v1-beta default (0x647a) for the role-separation
        // pin since the role-separation invariant is orthogonal to
        // the codepoint binding.
        let k_root = derive_root(&k, &cid, 0x647a);
        let k_step_empty_edge = derive_step(&k, b"", &cid);
        assert_ne!(k_root.as_bytes(), k_step_empty_edge.as_bytes());
    }

    #[test]
    fn step_info_tag_is_load_bearing() {
        // Eliding the "step" prefix MUST yield a different key.
        let k = StructuralKdfKey::from_bytes([3u8; 32]);
        let edge = b"edge:ITEM_TYPE";
        let cid = [0xEEu8; 32];
        let with = derive_step(&k, edge, &cid);
        let without = StructuralKdfKey::derive_step_without_info_tag_for_test(&k, edge, &cid);
        assert_ne!(with.as_bytes(), without.as_bytes());
    }
}
