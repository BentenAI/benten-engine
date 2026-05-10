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

impl JsKeypair {
    /// Crate-internal duplicator for cross-module use (e.g.
    /// `bindings/napi/src/atrium.rs::JsAtrium::set_local_device_keypair`
    /// per R6-FP Wave A Sub-A2).
    ///
    /// Per `crypto-blocker-1`, `benten_id::keypair::Keypair`
    /// deliberately does NOT implement `Clone` — secret bytes cannot
    /// be silently duplicated. The audited path is
    /// `Keypair::export_seed_envelope` +
    /// `Keypair::from_dag_cbor_envelope`; this helper encapsulates
    /// that round-trip so the napi `JsAtrium` setter can produce an
    /// owned `Keypair` for the engine-side `set_local_device_keypair`
    /// API. Not exposed across the napi boundary.
    pub(crate) fn duplicate_via_envelope(&self) -> Result<RustKeypair> {
        let envelope = self.inner.export_seed_envelope();
        RustKeypair::from_dag_cbor_envelope(&envelope)
            .map_err(|e| Error::from_reason(format!("keypair envelope re-import failed: {e}")))
    }
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

// ============================================================================
// G14-A2 wave-4a' — VC + DeviceAttestation napi surfaces
// ============================================================================
//
// Per Q1 alias-based standing rule: thin wrapping bridge — no
// `<B: GraphBackend>` cascade. Per CLAUDE.md baked-in #17, both
// classes stay native-only (`cfg(not(target_arch = "wasm32"))` is
// applied at the module level in `bindings/napi/src/lib.rs`).

use benten_id::device_attestation::{
    Acceptor as RustAcceptor, CapabilityEnvelope as RustCapabilityEnvelope,
    DeviceAttestation as RustDeviceAttestation, FreshnessPolicy as RustFreshnessPolicy,
    RuntimeTarget as RustRuntimeTarget, UptimePolicy as RustUptimePolicy,
    ZoneScope as RustZoneScope,
};
use benten_id::did::Did as RustDid;
use benten_id::vc::{
    Credential as RustCredential, TrustDomain as RustTrustDomain, verify_at as rust_vc_verify_at,
    verify_in_trust_domain as rust_vc_verify_in_trust_domain,
};

/// Verifiable Credential wrapper (G14-A2 wave-4a').
///
/// JavaScript surface — mirror of [`benten_id::vc::Credential`]:
///
/// ```js
/// const issuer = Keypair.generate();
/// const subjectDid = "did:key:z..."; // recipient
/// const vc = VerifiableCredential.issue(
///   issuer,
///   subjectDid,
///   "alumniOf",
///   "ExampleU",
///   1_000_000_000n,
///   1_000_086_400n, // optional exp
/// );
/// VerifiableCredential.verifyAt(vc, issuer.publicKeyDid(), 1_000_001_000n);
/// ```
///
/// Fields are not directly exposed; the `getClaimName` / `getClaimValue`
/// / `getIssuer` / `getSubject` accessors give read-only view into the
/// Rust-side claims payload.
#[napi(js_name = "VerifiableCredential")]
pub struct JsVerifiableCredential {
    inner: RustCredential,
}

#[napi]
impl JsVerifiableCredential {
    /// Issue a VC. `expiration_secs` may be `None` (no exp).
    #[napi(factory)]
    pub fn issue(
        issuer: &JsKeypair,
        subject_did: String,
        claim_name: String,
        claim_value: String,
        issuance_secs: i64,
        expiration_secs: Option<i64>,
    ) -> Result<Self> {
        let issuer_did = issuer.inner.public_key().to_did();
        let subject = RustDid::from_string_unchecked(subject_did);
        let mut builder = RustCredential::builder()
            .issuer(&issuer_did)
            .subject(&subject)
            .claim(claim_name, claim_value)
            .issued_at(issuance_secs.max(0) as u64);
        if let Some(exp) = expiration_secs {
            builder = builder.expires_at(exp.max(0) as u64);
        }
        let vc = builder
            .sign(&issuer.inner)
            .map_err(|e| Error::from_reason(format!("vc issuance failed: {e}")))?;
        Ok(Self { inner: vc })
    }

    /// Borrow the issuer DID string.
    #[napi]
    pub fn get_issuer(&self) -> String {
        self.inner.issuer().to_string()
    }

    /// Borrow the subject DID string.
    #[napi]
    pub fn get_subject(&self) -> String {
        self.inner.subject().to_string()
    }

    /// Borrow the claim name.
    #[napi]
    pub fn get_claim_name(&self) -> String {
        self.inner.claim().0.to_string()
    }

    /// Borrow the claim value.
    #[napi]
    pub fn get_claim_value(&self) -> String {
        self.inner.claim().1.to_string()
    }

    /// Verify at a given epoch second (rejects expired credentials).
    #[napi]
    pub fn verify_at(&self, expected_issuer_did: String, now_secs: i64) -> Result<bool> {
        let did = RustDid::from_string_unchecked(expected_issuer_did);
        match rust_vc_verify_at(&self.inner, &did, now_secs.max(0) as u64) {
            Ok(()) => Ok(true),
            Err(e) => Err(Error::from_reason(format!("vc verify_at failed: {e}"))),
        }
    }

    /// Verify under a trust-domain allow-list of issuer DIDs.
    #[napi]
    pub fn verify_in_trust_domain(&self, trusted_issuer_dids: Vec<String>) -> Result<bool> {
        let dids = trusted_issuer_dids
            .into_iter()
            .map(RustDid::from_string_unchecked)
            .collect::<Vec<_>>();
        let trust_domain = RustTrustDomain::new(dids);
        match rust_vc_verify_in_trust_domain(&self.inner, &trust_domain) {
            Ok(()) => Ok(true),
            Err(e) => Err(Error::from_reason(format!(
                "vc verify_in_trust_domain failed: {e}"
            ))),
        }
    }
}

/// Device-DID capability-attestation wrapper (G14-A2 wave-4a').
///
/// Per CLAUDE.md baked-in #17 + D-PHASE-3-25, the thin client uses
/// `engine.declareDeviceAttestation({...})` (TS surface in
/// `packages/engine/src/identity.ts`) to declare its envelope at
/// handshake time. The TS surface routes to this napi class on
/// full-peer Node.js targets.
///
/// Browser-target rejection at construction time: when
/// `runs_sandbox=true` is requested with `runtime_target="browser"`,
/// `issueWithRuntimeCheck` rejects with the typed catalog code
/// `E_DEVICE_ATTESTATION_INCOMPATIBLE_WITH_RUNTIME`.
#[napi(js_name = "DeviceAttestation")]
pub struct JsDeviceAttestation {
    inner: RustDeviceAttestation,
}

/// Convert TS-friendly envelope literals to the Rust shape.
fn envelope_from_str(
    runs_sandbox: bool,
    holds_zones: String,
    online_uptime: String,
    runs_atrium_peer: bool,
) -> Result<RustCapabilityEnvelope> {
    let holds_zones = match holds_zones.as_str() {
        "full" => RustZoneScope::Full,
        "cache_only" => RustZoneScope::CacheOnly,
        _ => {
            return Err(Error::from_reason(format!(
                "unknown holds_zones: {holds_zones}"
            )));
        }
    };
    let online_uptime = match online_uptime.as_str() {
        "always_on" => RustUptimePolicy::AlwaysOn,
        "session_bounded" => RustUptimePolicy::SessionBounded,
        _ => {
            return Err(Error::from_reason(format!(
                "unknown online_uptime: {online_uptime}"
            )));
        }
    };
    Ok(RustCapabilityEnvelope {
        runs_sandbox,
        holds_zones,
        online_uptime,
        runs_atrium_peer,
    })
}

impl JsDeviceAttestation {
    /// Crate-internal accessor for cross-module use (e.g.
    /// `bindings/napi/src/atrium.rs::JsAtrium::set_local_device_attestation`
    /// per R6-FP Wave A Sub-A2). Clones the inner Rust attestation so
    /// the caller can hand it to engine-side setters that take an owned
    /// `DeviceAttestation`. Not exposed across the napi boundary.
    pub(crate) fn inner_clone(&self) -> RustDeviceAttestation {
        self.inner.clone()
    }
}

#[napi]
impl JsDeviceAttestation {
    /// Issue a DeviceAttestation. The envelope shape mirrors
    /// `IdentityHandshake.envelope` in `packages/engine/src/identity.ts`.
    #[napi(factory)]
    pub fn issue(
        parent: &JsKeypair,
        device_did: String,
        runs_sandbox: bool,
        holds_zones: String,
        online_uptime: String,
        runs_atrium_peer: bool,
    ) -> Result<Self> {
        let envelope =
            envelope_from_str(runs_sandbox, holds_zones, online_uptime, runs_atrium_peer)?;
        let device = RustDid::from_string_unchecked(device_did);
        let attestation = RustDeviceAttestation::issue(&parent.inner, device, envelope)
            .map_err(|e| Error::from_reason(format!("attestation issuance failed: {e}")))?;
        Ok(Self { inner: attestation })
    }

    /// Issue with runtime-target check. `runtime_target` is `"browser"`
    /// or `"native"`. Rejects at construction time when browser +
    /// `runs_sandbox=true` per `br-r4-r1-4` / `br-r4-r2-3` MAJOR.
    #[napi(factory)]
    pub fn issue_with_runtime_check(
        parent: &JsKeypair,
        device_did: String,
        runs_sandbox: bool,
        holds_zones: String,
        online_uptime: String,
        runs_atrium_peer: bool,
        runtime_target: String,
    ) -> Result<Self> {
        let envelope =
            envelope_from_str(runs_sandbox, holds_zones, online_uptime, runs_atrium_peer)?;
        let device = RustDid::from_string_unchecked(device_did);
        let target = match runtime_target.as_str() {
            "browser" => RustRuntimeTarget::Browser,
            "native" => RustRuntimeTarget::Native,
            _ => {
                return Err(Error::from_reason(format!(
                    "unknown runtime_target: {runtime_target} (expected `browser`/`native`)"
                )));
            }
        };
        let attestation = RustDeviceAttestation::issue_with_runtime_check(
            &parent.inner,
            device,
            envelope,
            target,
        )
        .map_err(|e| Error::from_reason(format!("[{}] {e}", e.code())))?;
        Ok(Self { inner: attestation })
    }

    /// Convenience: issue browser-target minimum-capability envelope.
    #[napi(factory)]
    pub fn issue_for_browser_target(parent: &JsKeypair, device_did: String) -> Result<Self> {
        let device = RustDid::from_string_unchecked(device_did);
        let attestation = RustDeviceAttestation::issue_for_browser_target(&parent.inner, device)
            .map_err(|e| Error::from_reason(format!("attestation issuance failed: {e}")))?;
        Ok(Self { inner: attestation })
    }

    /// Borrow the device-DID string.
    #[napi]
    pub fn device_did(&self) -> String {
        self.inner.device_did.clone()
    }

    /// Borrow the parent-DID string.
    #[napi]
    pub fn parent_did(&self) -> String {
        self.inner.parent_did.clone()
    }

    /// Borrow the envelope's `runs_sandbox` field.
    #[napi]
    pub fn runs_sandbox(&self) -> bool {
        self.inner.envelope.runs_sandbox
    }

    /// Borrow the envelope's `runs_atrium_peer` field.
    #[napi]
    pub fn runs_atrium_peer(&self) -> bool {
        self.inner.envelope.runs_atrium_peer
    }

    /// Verify the attestation against the parent's `did:key` string.
    #[napi]
    pub fn verify_signature(&self, parent_did: String) -> Result<bool> {
        let did = RustDid::from_string_unchecked(parent_did);
        let pk = did
            .resolve()
            .map_err(|e| Error::from_reason(format!("invalid did:key: {e}")))?;
        match self.inner.verify_signature_with(&pk) {
            Ok(()) => Ok(true),
            Err(e) => Err(Error::from_reason(format!(
                "attestation verify failed: [{}] {e}",
                e.code()
            ))),
        }
    }

    /// Accept under a freshness policy (in seconds). Convenience
    /// wrapper around `Acceptor::accept_at` using `now_secs`.
    /// Returns `true` on accept; throws on rejection (carries the
    /// typed error code in the message).
    #[napi]
    pub fn accept_at(&self, now_secs: i64, freshness_window_secs: i64) -> Result<bool> {
        let acceptor = RustAcceptor::new(RustFreshnessPolicy::seconds(
            freshness_window_secs.max(0) as u64,
        ));
        match acceptor.accept_at(&self.inner, now_secs.max(0) as u64) {
            Ok(()) => Ok(true),
            Err(e) => Err(Error::from_reason(format!(
                "attestation reject: [{}] {e}",
                e.code()
            ))),
        }
    }
}
