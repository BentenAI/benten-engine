//! LAMPS-faithful spike: verify (and try to reproduce) the real IETF LAMPS WG
//! `id-MLDSA65-Ed25519-SHA512` composite KAT vector using Benten's pinned
//! `ml_dsa 0.1.0` + `ed25519-dalek 2.2` + `sha2 0.10`.
//!
//! Construction under test (draft-ietf-lamps-pq-composite-sigs-19):
//!   M'      = Prefix || Label || len(ctx) || ctx || SHA-512(M)
//!   Prefix  = "CompositeAlgorithmSignatures2025"             (32 B)
//!   Label   = "COMPSIG-MLDSA65-Ed25519-SHA512"               (30 B)
//!   mldsaSig = ML-DSA.Sign(mldsaSK, M', mldsa_ctx = Label)   <-- ctx = Label!
//!   tradSig  = Ed25519.Sign(tradSK, M')                      <-- no ctx
//!   composite sig = mldsaSig(3309) || tradSig(64)
//!   composite pk  = mldsaPK(1952)  || tradPK(32)

use ed25519_dalek::{Signature as EdSig, Verifier, VerifyingKey as EdVk};
use ml_dsa::{
    EncodedSignature, EncodedVerifyingKey, MlDsa65, Seed, Signature as MlSig, SigningKey as MlSk,
    VerifyingKey as MlVk,
};
use sha2::{Digest, Sha512};

const PREFIX: &[u8] = b"CompositeAlgorithmSignatures2025";
const LABEL: &[u8] = b"COMPSIG-MLDSA65-Ed25519-SHA512";

// ---- Embedded real LAMPS vector bytes (decoded from testvectors.json, commit
//      f0627ab3..., tcId id-MLDSA65-Ed25519-SHA512). Filled at runtime from the
//      downloaded JSON to avoid 8KB of hardcoded hex; see fn load_vector below. ----

fn m_prime(ctx: &[u8], msg: &[u8]) -> Vec<u8> {
    let ph = Sha512::digest(msg);
    let mut out = Vec::new();
    out.extend_from_slice(PREFIX);
    out.extend_from_slice(LABEL);
    out.push(u8::try_from(ctx.len()).expect("ctx <= 255"));
    out.extend_from_slice(ctx);
    out.extend_from_slice(&ph);
    out
}

struct Vector {
    pk: Vec<u8>,  // 1984 = mldsaPK(1952)||tradPK(32)
    sk: Vec<u8>,  // 64   = mldsaSeed(32)||tradSeed(32)
    m: Vec<u8>,
    ctx: Vec<u8>,
    s_empty: Vec<u8>,       // composite sig over empty ctx
    s_with_ctx: Vec<u8>,    // composite sig over the non-empty global ctx
}

fn load_vector() -> Vector {
    // Minimal JSON pull without serde: read the file, base64-decode the fields
    // we extracted with python. We hardcode the base64 strings here (verbatim
    // from testvectors.json) so the spike is self-contained.
    let raw = std::fs::read_to_string("testvectors.json")
        .expect("run from .tmp-lamps-scratch/ with testvectors.json present");
    let v: serde_like::Json = serde_like::parse(&raw);
    let m = b64(v.global_str("m"));
    let ctx = b64(v.global_str("ctx"));
    let t = v.test("id-MLDSA65-Ed25519-SHA512");
    Vector {
        pk: b64(&t["pk"]),
        sk: b64(&t["sk"]),
        m,
        ctx,
        s_empty: b64(&t["s"]),
        s_with_ctx: b64(&t["sWithContext"]),
    }
}

fn b64(s: &str) -> Vec<u8> {
    // Standard base64 decode (RFC 4648, '+' '/' '=' padding).
    const T: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut inv = [255u8; 256];
    for (i, &c) in T.iter().enumerate() {
        inv[c as usize] = i as u8;
    }
    let mut bits = 0u32;
    let mut nbits = 0;
    let mut out = Vec::new();
    for &c in s.as_bytes() {
        if c == b'=' || c == b'\n' || c == b'\r' {
            continue;
        }
        let val = inv[c as usize];
        assert!(val != 255, "bad b64 char {c}");
        bits = (bits << 6) | val as u32;
        nbits += 6;
        if nbits >= 8 {
            nbits -= 8;
            out.push((bits >> nbits) as u8);
        }
    }
    out
}

fn main() {
    let v = load_vector();
    println!("=== LAMPS-faithful spike (ml_dsa 0.1.0, ed25519-dalek 2.2, sha2 0.10) ===");
    println!("pk len={} sk len={} s len={} sWithContext len={}", v.pk.len(), v.sk.len(), v.s_empty.len(), v.s_with_ctx.len());
    println!("m={:?}", String::from_utf8_lossy(&v.m));
    println!("global ctx len={}", v.ctx.len());

    let mldsa_pk = &v.pk[..1952];
    let trad_pk = &v.pk[1952..1984];

    // ============ MUST-HAVE: VERIFY the published vector (empty-ctx s) ============
    println!("\n--- VERIFY published vector (empty ctx, field 's') ---");
    let mp_empty = m_prime(b"", &v.m);
    let mldsa_sig = &v.s_empty[..3309];
    let trad_sig = &v.s_empty[3309..3373];

    // Ed25519 half: verify M' (no ctx).
    let ed_vk = EdVk::from_bytes(trad_pk.try_into().unwrap()).expect("ed25519 pk valid");
    let ed_sig = EdSig::from_bytes(trad_sig.try_into().unwrap());
    let ed_ok = ed_vk.verify(&mp_empty, &ed_sig).is_ok();
    println!("  Ed25519 half verify (M', no ctx): {}", yn(ed_ok));

    // ML-DSA half: verify_with_context(M', Label, sig).
    let ml_vk_enc = EncodedVerifyingKey::<MlDsa65>::try_from(mldsa_pk).expect("mldsa pk encoding");
    let ml_vk = MlVk::<MlDsa65>::decode(&ml_vk_enc);
    let ml_sig_enc = EncodedSignature::<MlDsa65>::try_from(mldsa_sig).expect("mldsa sig encoding");
    let ml_sig = MlSig::<MlDsa65>::decode(&ml_sig_enc).expect("mldsa sig decode");
    let ml_ok_label = ml_vk.verify_with_context(&mp_empty, LABEL, &ml_sig);
    println!("  ML-DSA half verify_with_context(M', ctx=Label): {}", yn(ml_ok_label));

    // Control: prove ctx=Label is load-bearing (empty ctx should FAIL).
    let ml_ok_emptyctx = ml_vk.verify_with_context(&mp_empty, b"", &ml_sig);
    println!("  [control] ML-DSA half verify_with_context(M', ctx=\"\"): {} (expect NO)", yn(ml_ok_emptyctx));

    println!("  COMPOSITE (empty ctx) BOTH HALVES VERIFY: {}", yn(ed_ok && ml_ok_label));

    // ============ also verify the non-empty-ctx variant (sWithContext) ============
    println!("\n--- VERIFY published vector (non-empty ctx, field 'sWithContext') ---");
    let mp_ctx = m_prime(&v.ctx, &v.m);
    let mldsa_sig_c = &v.s_with_ctx[..3309];
    let trad_sig_c = &v.s_with_ctx[3309..3373];
    let ed_sig_c = EdSig::from_bytes(trad_sig_c.try_into().unwrap());
    let ed_ok_c = ed_vk.verify(&mp_ctx, &ed_sig_c).is_ok();
    let ml_sig_c_enc = EncodedSignature::<MlDsa65>::try_from(mldsa_sig_c).expect("mldsa sig encoding");
    let ml_sig_c = MlSig::<MlDsa65>::decode(&ml_sig_c_enc).expect("mldsa sig decode");
    let ml_ok_c = ml_vk.verify_with_context(&mp_ctx, LABEL, &ml_sig_c);
    println!("  Ed25519 half verify (M' with ctx, no ed-ctx): {}", yn(ed_ok_c));
    println!("  ML-DSA half verify_with_context(M' with ctx, ctx=Label): {}", yn(ml_ok_c));
    println!("  COMPOSITE (non-empty ctx) BOTH HALVES VERIFY: {}", yn(ed_ok_c && ml_ok_c));

    // ============ NICE-TO-HAVE: BYTE-REPRODUCE from seeds ============
    println!("\n--- BYTE-REPRODUCE from seeds (sk = mldsaSeed(32)||tradSeed(32)) ---");
    let mldsa_seed = &v.sk[..32];
    let trad_seed = &v.sk[32..64];

    // Ed25519: signing key from 32-byte seed; verify it derives the same pk.
    let ed_sk = ed25519_dalek::SigningKey::from_bytes(trad_seed.try_into().unwrap());
    let derived_ed_pk = ed_sk.verifying_key();
    println!("  Ed25519 derived pk matches vector tradPK: {}", yn(derived_ed_pk.to_bytes() == trad_pk));
    // Ed25519 is deterministic (RFC 8032), so re-signing M' must byte-match.
    use ed25519_dalek::Signer as _;
    let repro_ed_sig: EdSig = ed_sk.sign(&mp_empty);
    println!("  Ed25519 re-sign(M') byte-matches vector tradSig: {}", yn(repro_ed_sig.to_bytes() == trad_sig));

    // ML-DSA: signing key from 32-byte seed; verify derives same pk.
    let seed = Seed::try_from(mldsa_seed).expect("32-byte ml-dsa seed");
    let ml_sk = MlSk::<MlDsa65>::from_seed(&seed);
    let derived_ml_vk = {
        use ml_dsa::signature::Keypair as _;
        ml_sk.verifying_key()
    };
    let derived_ml_pk_bytes = derived_ml_vk.encode();
    println!("  ML-DSA derived pk matches vector mldsaPK: {}", yn(derived_ml_pk_bytes.as_slice() == mldsa_pk));

    // ML-DSA deterministic sign with ctx=Label, via the ExpandedSigningKey path.
    let esk = ml_sk.expanded_key();
    let repro_ml_sig = esk
        .sign_deterministic(&mp_empty, LABEL)
        .expect("sign_deterministic ctx<=255");
    let repro_ml_bytes = repro_ml_sig.encode();
    let ml_byte_match = repro_ml_bytes.as_slice() == mldsa_sig;
    println!("  ML-DSA sign_deterministic(M', ctx=Label) byte-matches vector mldsaSig: {}", yn(ml_byte_match));
    if !ml_byte_match {
        // Determine whether the vector used the hedged (randomized) variant.
        // Even if not byte-identical, the deterministic sig MUST self-verify.
        let self_ok = derived_ml_vk.verify_with_context(&mp_empty, LABEL, &repro_ml_sig);
        println!("    (note) deterministic re-sign differs => vector likely HEDGED/randomized; our deterministic sig self-verifies: {}", yn(self_ok));
    }

    println!("\n=== SUMMARY ===");
    println!("PUBLISHED VECTOR VERIFY (empty ctx): {}", yn(ed_ok && ml_ok_label));
    println!("PUBLISHED VECTOR VERIFY (non-empty ctx): {}", yn(ed_ok_c && ml_ok_c));
    println!("ctx=Label is load-bearing (empty-ctx ML-DSA verify fails): {}", yn(!ml_ok_emptyctx));
    println!("BYTE-REPRODUCE Ed25519: {}", yn(repro_ed_sig.to_bytes() == trad_sig));
    println!("BYTE-REPRODUCE ML-DSA: {}", yn(ml_byte_match));
}

fn yn(b: bool) -> &'static str {
    if b { "YES" } else { "NO" }
}

// --- tiny hand-rolled JSON reader (avoid adding serde_json dep) ---
mod serde_like {
    pub struct Json {
        text: String,
    }
    pub fn parse(s: &str) -> Json {
        Json { text: s.to_string() }
    }
    impl Json {
        pub fn global_str(&self, key: &str) -> &str {
            // matches  "key": "value"  at top scope; m and ctx are unique enough.
            find_string_value(&self.text, key)
        }
        pub fn test(&self, tc: &str) -> std::collections::HashMap<String, String> {
            // Find the object containing  "tcId": "<tc>"  then collect the
            // string fields pk/sk/s/sWithContext that follow within that object.
            let needle = format!("\"tcId\": \"{tc}\"");
            let start = self.text.find(&needle).expect("tcId present");
            // object spans from the preceding '{' to the matching '}'.
            let obj_start = self.text[..start].rfind('{').unwrap();
            let mut depth = 0i32;
            let mut end = obj_start;
            for (i, c) in self.text[obj_start..].char_indices() {
                match c {
                    '{' => depth += 1,
                    '}' => {
                        depth -= 1;
                        if depth == 0 {
                            end = obj_start + i + 1;
                            break;
                        }
                    }
                    _ => {}
                }
            }
            let obj = &self.text[obj_start..end];
            let mut map = std::collections::HashMap::new();
            for key in ["pk", "sk", "s", "sWithContext"] {
                map.insert(key.to_string(), find_string_value(obj, key).to_string());
            }
            map
        }
    }
    fn find_string_value<'a>(hay: &'a str, key: &str) -> &'a str {
        let pat = format!("\"{key}\":");
        let mut search_from = 0;
        loop {
            let rel = hay[search_from..].find(&pat).expect("key present");
            let kpos = search_from + rel;
            // ensure this is a real key boundary (preceded by whitespace/{/,)
            let after = &hay[kpos + pat.len()..];
            let after = after.trim_start();
            if let Some(rest) = after.strip_prefix('"') {
                let endq = rest.find('"').unwrap();
                return &rest[..endq];
            }
            search_from = kpos + pat.len();
        }
    }
}
