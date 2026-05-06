//! G14-A1 wave-4a — napi bridge for `benten-id` identity primitives.
//!
//! Surfaces `Keypair`, `Did`, and `UcanClaim` to JavaScript callers
//! through the napi-rs v3 binding layer.
//!
//! ## Deployment-shape gating (CLAUDE.md baked-in #17)
//!
//! The napi cdylib is the native full-peer entry point. Browser tab
//! thin-client builds (`wasm32-unknown-unknown`) consume the same
//! conceptual surface but through `packages/engine/src/identity.ts` —
//! which carries the type declarations only. The `wasm32` deployment
//! shape does NOT pull `benten-id` cryptographic operations into the
//! browser bundle (Loro / iroh / SANDBOX / direct-sync state are
//! native-only per #17; identity primitives stay on the full-peer
//! side and the thin client declares its identity envelope to the
//! full peer at handshake).
//!
//! Gated with `#[cfg(all(feature = "napi-export", not(target_arch = "wasm32")))]`
//! per the same pattern as the WAIT / STREAM / SUBSCRIBE adapters.
//!
//! ## Q1 standing rule (alias-based pragmatic-genericism)
//!
//! This module is a THIN WRAPPING BRIDGE — no `<B: GraphBackend>`
//! generic cascade. `benten-id` is upstream of the GraphBackend
//! umbrella trait (per `arch-r1-10`); cascading would invert the
//! dependency layering.

use napi::bindgen_prelude::*;
use napi_derive::napi;

use benten_id::keypair::{Keypair as RustKeypair, Signature as RustSignature};

/// Ed25519 keypair wrapper.
///
/// JavaScript surface:
///
/// ```js
/// const kp = Keypair.generate();
/// const did = kp.publicKeyDid();   // -> "did:key:z..."
/// const sig = kp.sign(Buffer.from("hello"));
/// ```
///
/// The secret bytes never cross the napi boundary in this surface.
/// Future extension paths (DAG-CBOR envelope export for backup) will
/// expose `exportSeedEnvelope()` returning `Buffer`; G14-A1 keeps
/// the surface minimal.
#[napi(js_name = "Keypair")]
pub struct JsKeypair {
    inner: RustKeypair,
}

#[napi]
impl JsKeypair {
    /// Generate a fresh keypair from the OS CSPRNG.
    ///
    /// Per `crypto-major-2`, this path is pinned to `OsRng`; never a
    /// deterministic seed.
    #[napi(factory)]
    pub fn generate() -> Self {
        Self {
            inner: RustKeypair::generate(),
        }
    }

    /// The public key as a `did:key:z<base58btc>` string.
    #[napi]
    pub fn public_key_did(&self) -> String {
        self.inner.public_key().to_did().as_str().to_string()
    }

    /// Sign a message with this keypair's secret. Returns the 64-byte
    /// Ed25519 signature.
    #[napi]
    pub fn sign(&self, message: Buffer) -> Buffer {
        let sig = self.inner.sign(message.as_ref());
        Buffer::from(sig.to_bytes().to_vec())
    }
}

/// Verify an Ed25519 signature given a `did:key:z<...>` issuer DID,
/// message bytes, and the 64-byte signature. Returns `true` if
/// verification succeeds, `false` otherwise.
///
/// Surface mirrors the TS side at
/// `packages/engine/src/identity.ts::verifySignature` — both go through
/// the same `did:key` resolution + Ed25519 verify path.
#[napi]
pub fn verify_signature(issuer_did: String, message: Buffer, signature: Buffer) -> Result<bool> {
    use benten_id::did::Did;
    let did = Did::from_string_unchecked(issuer_did);
    let pk = match did.resolve() {
        Ok(pk) => pk,
        Err(e) => return Err(Error::from_reason(format!("invalid did:key: {e}"))),
    };
    let sig_bytes: [u8; 64] = match signature.as_ref().try_into() {
        Ok(b) => b,
        Err(_) => {
            return Err(Error::from_reason(
                "signature must be exactly 64 bytes (Ed25519)",
            ));
        }
    };
    let sig = RustSignature::from_bytes(&sig_bytes);
    Ok(pk.verify(message.as_ref(), &sig).is_ok())
}

// Note: full UCAN builder + chain-walk surface stays Rust-only at
// G14-A1. The TS handshake surface (`packages/engine/src/identity.ts`)
// declares the shape so the thin-client deployment shape can compose
// claims; the validate path runs on the full peer's native side.
