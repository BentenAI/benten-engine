/**
 * Phase 3 G14-A1 wave-4a — TS surface for identity primitives.
 *
 * ## Deployment-shape commitment (CLAUDE.md baked-in #17)
 *
 * Two deployment shapes consume this module:
 *
 * 1. **Full peer (native)** — Node.js host links the napi cdylib via
 *    `@benten/engine`. The {@link Keypair} / {@link Did} / signing /
 *    verifying operations route into `bindings/napi/src/identity.rs`
 *    where the real Ed25519 cryptography lives.
 * 2. **Thin compute surface (browser tab; Phase-9+ edge worker)** —
 *    the wasm32 deployment shape DOES NOT pull `benten-id` into the
 *    bundle (Loro / iroh / SANDBOX / direct-sync state are native-
 *    only per #17; identity primitives stay on the full-peer side).
 *    On thin clients, the {@link IdentityHandshake} type is the
 *    surface the thin client uses to declare its envelope to the
 *    full peer at handshake time. The full peer signs / verifies on
 *    behalf of the thin client.
 *
 * ## Q1 standing rule (alias-based pragmatic-genericism)
 *
 * This module declares only TYPES + thin pass-through helpers. No
 * generic cascade. The runtime crypto path is in Rust.
 *
 * @packageDocumentation
 */

/**
 * `did:key` DID string.
 *
 * Per W3C did-method-key spec (`https://w3c-ccg.github.io/did-method-key/`):
 * `did:key:z<base58btc(0xed01 || <32 pubkey bytes>)>`. Multibase prefix
 * `z` (base58btc) + multicodec varint `0xed01` (Ed25519).
 */
export type Did = string & { readonly __brand: "did:key" };

/**
 * UCAN capability — `(resource, ability)` pair.
 *
 * Examples: `{ resource: "/zone/posts", ability: "read" }`,
 * `{ resource: "host:sandbox:exec", ability: "*" }`.
 */
export interface Capability {
  resource: string;
  ability: string;
}

/**
 * UCAN claim payload.
 *
 * - `iss` — issuer DID (the keypair that signed)
 * - `aud` — audience DID (the recipient this claim is delegated to)
 * - `att` — capabilities granted
 * - `nbf` / `exp` — not-before / expiration epoch seconds (per
 *   `crypto-blocker-2`)
 * - `prf` — proof chain (parent UCANs whose authority this derives
 *   from)
 */
export interface UcanClaims {
  iss: Did;
  aud: Did;
  att: Capability[];
  nbf?: number;
  exp?: number;
  prf?: SignedUcan[];
}

/**
 * Signed UCAN — claims + Ed25519 signature over the
 * canonical-DAG-CBOR encoding of the claims.
 */
export interface SignedUcan {
  claims: UcanClaims;
  /** Ed25519 signature, hex-encoded (64 raw bytes). */
  signature: string;
}

/**
 * Identity handshake envelope — declared by a thin compute surface
 * (browser tab / edge worker) to its full-peer host at session start.
 *
 * Per CLAUDE.md baked-in #17, the thin client's envelope advertises:
 *
 * - `runs_sandbox: false` — wasmtime is unavailable on
 *   wasm32-unknown-unknown.
 * - `holds_zones: "cache_only"` — read-only snapshot view.
 * - `runs_atrium_peer: false` — sync runs on the full peer.
 * - `online_uptime: "session_bounded"` — tab close ends participation.
 *
 * The full peer's UCAN backend consumes this envelope at delegation
 * chain-walk so per-device cap policy can enforce envelope-derived
 * limits. (G14-A2 wave-4a' wires the device-DID attestation surface
 * that turns this declaration into a signed claim.)
 */
export interface IdentityHandshake {
  did: Did;
  envelope: {
    runs_sandbox: boolean;
    holds_zones: "full" | "cache_only" | { specific: string[] };
    runs_atrium_peer: boolean;
    online_uptime: "always_on" | "session_bounded";
  };
}

/**
 * Browser thin-client envelope literal — minimum-capability
 * declaration consistent with CLAUDE.md baked-in #17.
 *
 * Use this at thin-client startup:
 *
 * ```ts
 * const handshake: IdentityHandshake = {
 *   did: myBrowserDid,
 *   envelope: BROWSER_THIN_CLIENT_ENVELOPE,
 * };
 * ```
 */
export const BROWSER_THIN_CLIENT_ENVELOPE: IdentityHandshake["envelope"] = {
  runs_sandbox: false,
  holds_zones: "cache_only",
  runs_atrium_peer: false,
  online_uptime: "session_bounded",
};

/**
 * Keypair handle — present only on the full-peer (native) deployment.
 *
 * On the thin compute surface (browser tab / edge worker), constructing
 * a `Keypair` throws — identity operations route to the full peer over
 * the authenticated thin-client protocol (D-PHASE-3-30).
 *
 * Per `pim-2-ts-canary` §3.6b amendment, the RED-PHASE behavior on
 * thin-client targets uses `throw new Error("RED-PHASE: ...")` — the
 * production path lands at G14-A2 with the device-DID attestation
 * surface.
 */
export interface KeypairHandle {
  /**
   * The `did:key:z<...>` representation of the public key.
   */
  publicKeyDid(): Did;

  /**
   * Sign a message with this keypair's secret. Returns the 64-byte
   * Ed25519 signature.
   */
  sign(message: Uint8Array): Uint8Array;
}

/**
 * Construct a fresh keypair from the OS CSPRNG (full-peer only).
 *
 * Routes to the napi binding `Keypair.generate()` at
 * `bindings/napi/src/identity.rs::JsKeypair::generate`. On thin-client
 * targets this entry point throws — see the module-level commitment
 * notes.
 *
 * @returns a {@link KeypairHandle} backed by the native Ed25519 surface.
 *
 * @throws on thin-client (browser tab / edge worker) targets per
 *   pim-2-ts-canary §3.6b — identity operations route to the full peer
 *   via the authenticated thin-client protocol (D-PHASE-3-30).
 */
export function generateKeypair(): KeypairHandle {
  // The actual implementation links `Keypair.generate()` from the
  // napi cdylib at runtime. Wired here as a TYPE-only declaration so
  // the thin-client bundle does not pull `benten-id` into the
  // wasm32 build artifact. Production wiring (calling into the
  // imported napi `Keypair` class) lands at G14-A2 alongside the
  // full handshake protocol surface.
  throw new Error(
    "RED-PHASE: G14-A2 wires generateKeypair() into the napi cdylib for full-peer targets; " +
      "thin-client targets route identity operations to the full peer via D-PHASE-3-30 protocol",
  );
}

/**
 * Verify an Ed25519 signature given an issuer DID + message + 64-byte
 * signature. Routes to the napi binding's `verifySignature` on
 * full-peer targets; on thin-client targets, routes to the full peer
 * over the authenticated protocol.
 */
export function verifySignature(
  _issuerDid: Did,
  _message: Uint8Array,
  _signature: Uint8Array,
): boolean {
  throw new Error(
    "RED-PHASE: G14-A2 wires verifySignature() into the napi cdylib for full-peer targets; " +
      "thin-client targets route identity operations to the full peer via D-PHASE-3-30 protocol",
  );
}

// ============================================================================
// G14-A2 wave-4a' — VC + DeviceAttestation type surfaces
// ============================================================================
//
// Per pim-2-ts-canary §3.6b amendment, RED-PHASE TypeScript pins use
// `throw new Error("RED-PHASE: <reason>")` (NOT `unimplemented!()`
// Rust analog). The runtime crypto path lives at
// `bindings/napi/src/identity.rs` (full-peer); thin-client targets
// route identity operations to the full peer over the authenticated
// thin-client protocol (D-PHASE-3-30).

/**
 * W3C Verifiable Credential v1.1 view (G14-A2). Mirrors
 * `benten_id::vc::Credential` on the Rust side.
 *
 * Issued via {@link issueVerifiableCredential} / verified via
 * {@link verifyVerifiableCredentialAt}. The signing path runs on the
 * full peer (napi); the thin client receives this object as a typed
 * view + delegates verification to the full peer over the
 * authenticated thin-client protocol.
 */
export interface VerifiableCredential {
  /** Issuer DID (signs the VC). */
  issuer: Did;
  /** Subject DID (the credential applies to). */
  subject: Did;
  /** Single claim — name + value. */
  claim: { name: string; value: string };
  /** Issuance epoch seconds. */
  issuanceDate: number;
  /** Optional expiration epoch seconds. */
  expirationDate?: number;
}

/**
 * Issue a VC on the full peer via the napi binding.
 *
 * @throws on thin-client (browser tab / edge worker) targets per
 *   pim-2-ts-canary §3.6b — VC issuance routes to the full peer via
 *   D-PHASE-3-30 protocol.
 */
export function issueVerifiableCredential(
  _issuer: KeypairHandle,
  _subject: Did,
  _claimName: string,
  _claimValue: string,
  _issuanceDate: number,
  _expirationDate?: number,
): VerifiableCredential {
  throw new Error(
    "RED-PHASE: G14-A2 wires issueVerifiableCredential() into the napi cdylib for full-peer targets; " +
      "thin-client targets route VC issuance to the full peer via D-PHASE-3-30 protocol",
  );
}

/**
 * Verify a VC at a given epoch second (rejects expired credentials).
 *
 * @throws on thin-client targets per pim-2-ts-canary §3.6b.
 */
export function verifyVerifiableCredentialAt(
  _vc: VerifiableCredential,
  _expectedIssuer: Did,
  _now: number,
): boolean {
  throw new Error(
    "RED-PHASE: G14-A2 wires verifyVerifiableCredentialAt() into the napi cdylib for full-peer targets; " +
      "thin-client targets route VC verification to the full peer via D-PHASE-3-30 protocol",
  );
}

/**
 * Device-DID capability-attestation view (G14-A2). Mirrors
 * `benten_id::device_attestation::DeviceAttestation` on the Rust side.
 *
 * Per CLAUDE.md baked-in #17 + D-PHASE-3-25, the thin client uses
 * {@link declareDeviceAttestation} to declare its envelope at
 * handshake time. The full peer signs + persists the attestation;
 * the chain-walker consults the envelope at UCAN delegation time so
 * per-device cap policy can enforce envelope-derived limits.
 */
export interface DeviceAttestation {
  deviceDid: Did;
  parentDid: Did;
  envelope: IdentityHandshake["envelope"];
}

/**
 * Declare a device-DID attestation (full-peer surface; thin client
 * delegates).
 *
 * Per `br-r4-r1-4` / `br-r4-r2-3` MAJOR, when the runtime is
 * `browser` and the envelope claims `runs_sandbox=true`, the
 * underlying napi factory throws with the typed catalog code
 * `E_DEVICE_ATTESTATION_INCOMPATIBLE_WITH_RUNTIME`. The trust graph
 * never receives a forged envelope.
 *
 * @throws on thin-client targets per pim-2-ts-canary §3.6b.
 */
export function declareDeviceAttestation(_handshake: IdentityHandshake): DeviceAttestation {
  // Full-peer code links the napi binding's `DeviceAttestation.issue(...)`
  // factory at runtime. Thin-client builds reach this surface and route
  // to the full peer over the authenticated thin-client protocol
  // (D-PHASE-3-30) per CLAUDE.md baked-in #17.
  throw new Error(
    "RED-PHASE: G14-A2 wires declareDeviceAttestation() into the napi cdylib for full-peer targets; " +
      "thin-client targets route attestation issuance to the full peer via D-PHASE-3-30 protocol",
  );
}
