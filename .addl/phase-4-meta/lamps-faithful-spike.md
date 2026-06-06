# LAMPS-faithful signature spike — verdict

**Goal.** De-risk re-implementing Benten's v1-beta signature default as the
BYTE-FAITHFUL IETF LAMPS composite `id-MLDSA65-Ed25519-SHA512`
(OID `1.3.6.1.5.5.7.6.48`), replacing the current "LAMPS-aligned" Benten NF-4
construction. Ben ratified this direction (option A) 2026-06-05.

**Method (reproduce-don't-assume).** A detached scratch crate in this worktree
(`.tmp-lamps-scratch/`, `[workspace]`-detached, deps = `ml-dsa 0.1`,
`ed25519-dalek 2.2`, `sha2 0.10` matching the workspace pins) downloaded the REAL
IETF LAMPS WG known-answer test vector (`lamps-wg/draft-composite-sigs`
`src/testvectors.json` @ commit `f0627ab34acfe1aee0abce4bee91ed2b577eab76`, the
same commit the in-tree `f_kat_4` fixture cites), decoded the
`id-MLDSA65-Ed25519-SHA512` case (full composite `pk`/`sk`-seeds/`s`/
`sWithContext`), and ran real cryptographic verification + reproduction. Base
branch `phase-4-meta-core/f-kat4-inbound` @ `4f35bc1c`.

---

## PUBLISHED VECTOR VERIFY: **YES** (both halves, both ctx variants)

Using Benten's own pinned primitives, the REAL published composite signature
verifies end-to-end:

```
--- VERIFY published vector (empty ctx, field 's') ---
  Ed25519 half verify (M', no ctx):                 YES
  ML-DSA half verify_with_context(M', ctx=Label):   YES
  [control] ML-DSA half verify_with_context(M', ""): NO  (expect NO)
  COMPOSITE (empty ctx) BOTH HALVES VERIFY:         YES

--- VERIFY published vector (non-empty ctx, field 'sWithContext') ---
  Ed25519 half verify (M' with ctx, no ed-ctx):       YES
  ML-DSA half verify_with_context(M' w/ctx, ctx=Label): YES
  COMPOSITE (non-empty ctx) BOTH HALVES VERIFY:        YES
```

The cross-checks also confirm: the embedded in-tree `real_lamps_vector::ED25519_PK`
and `ED25519_SIG` are byte-identical to `pk[1952..1984]` / `s[3309..3373]` of the
downloaded vector (the in-tree fixture is authentic).

### The exact M' recipe that made it work (confirmed byte-exact against the real vector)

```
M'      = Prefix || Label || len(ctx) || ctx || SHA-512(M)
Prefix  = "CompositeAlgorithmSignatures2025"   (32 bytes; ASCII)
Label   = "COMPSIG-MLDSA65-Ed25519-SHA512"     (30 bytes; ASCII)
len(ctx)= single u8 (0..=255)
PH      = SHA-512  (computed ONCE; shared by both halves)

mldsaSig = ML-DSA-65.Sign(mldsaSK, M', mldsa_ctx = Label)   <-- ctx = the Label!
tradSig  = Ed25519.Sign(tradSK, M')                          <-- M', no ctx
composite_sig = mldsaSig(3309) || tradSig(64)                = 3373 B (raw concat, NO ASN.1)
composite_pk  = mldsaPK(1952)  || tradPK(32)                 = 1984 B (raw concat)
```

**Load-bearing finding — `mldsa_ctx = Label`, not empty.** The control line proves
it: feeding `ctx=""` to the ML-DSA verify FAILS, feeding `ctx=Label` SUCCEEDS. This
is the single most important construction detail and it is NOT what the prior
in-tree `f_kat_4` module comment implied (the comment's "`... || 0x00 || SHA-512(m)`"
is right for the *Ed25519* half but the *ML-DSA* half is invoked with
`ctx = Label`, which makes ML-DSA internally prepend its own
`0x00 || len(Label) || Label` domain-separator to `M'`). The byte-faithful re-impl
MUST pass the Label as the ML-DSA context. (`draft-19` Step 4:
`mldsaSig = ML-DSA.Sign(mldsaSK, M', mldsa_ctx=Label)`; underlying ML-DSA is
**pure** ML-DSA, not `Sign_internal`. Confirmed against both the datatracker
draft-19 text and the lamps-wg repo `main` markdown.)

---

## BYTE-REPRODUCE: **PARTIAL — Ed25519 YES, ML-DSA N/A-by-design (vector is hedged)**

```
--- BYTE-REPRODUCE from seeds (sk = mldsaSeed(32)||tradSeed(32)) ---
  Ed25519 derived pk matches vector tradPK:                     YES
  Ed25519 re-sign(M') byte-matches vector tradSig:              YES
  ML-DSA derived pk matches vector mldsaPK:                     YES
  ML-DSA sign_deterministic(M', ctx=Label) byte-matches sig:    NO
    -> our deterministic re-sign self-verifies:                 YES
```

- **Ed25519 byte-reproduces exactly** (RFC-8032 deterministic). Derived pk from the
  32-byte seed matches, and re-signing `M'` reproduces the vector's `tradSig`
  byte-for-byte.
- **ML-DSA does NOT byte-reproduce, and that is CORRECT, not a failure.** ML-DSA-65
  derived-pk-from-seed matches the vector's `mldsaPK` (so the seed→key path is
  right). The signature differs only because the published vector was generated
  with the **hedged (randomized) `rnd` variant**, whereas we re-signed with the
  **deterministic** variant (`rnd = 0`). LAMPS draft-19 leaves hedged-vs-deterministic
  as an implementation choice (no `rnd` is prescribed in Step 4); both produce valid
  signatures any verifier accepts. Our deterministic re-sign self-verifies. So
  "byte-reproduce" of the ML-DSA half against THIS vector is **N/A by design** — the
  vector isn't a deterministic KAT for the ML-DSA leg. (The ML-DSA *verification* of
  the published bytes is the real KAT, and it passes — see above.)

---

## ml_dsa API for LAMPS M'/context: **AVAILABLE in the pinned `ml-dsa 0.1.0`** ✅

This was THE load-bearing question. **`ml-dsa 0.1.0` exposes everything the
byte-faithful LAMPS composite needs.** The pure-`Signer::sign(msg)` path Benten
currently uses is only the convenience trait impl (empty-ctx); the context-bearing
methods exist and are `pub`:

Verify side (`ml_dsa::VerifyingKey<MlDsa65>`):
- `pub fn verify_with_context(&self, M: &[u8], ctx: &[u8], sigma: &Signature<P>) -> bool`
  — exactly FIPS-204 Alg. 3 (pure ML-DSA with a context string). `verify_with_context(M', Label, sig)` is the LAMPS ML-DSA verify. (`src/verifying.rs:131`.)

Sign side (`ml_dsa::ExpandedSigningKey<MlDsa65>`, reached via
`SigningKey::expanded_key()` which is `pub` though `#[doc(hidden)]`):
- `pub fn sign_deterministic(&self, M: &[u8], ctx: &[u8]) -> Result<Signature<P>, Error>`
  (`src/signing.rs:428`) — the deterministic ML-DSA.Sign with a context string.
- `pub fn sign_randomized<R: TryCryptoRng>(&self, M, ctx, rng) -> Result<Signature, Error>`
  (`src/signing.rs:377`, behind the already-enabled `rand_core` feature) — the
  **hedged** variant, matching how the published vector was produced.
- `sign_mu_deterministic` / `sign_mu_randomized` / `sign_internal` also exist (μ-precomputed / internal forms; not needed for LAMPS).

Both `ctx`-bearing methods enforce `ctx.len() <= 255` (the LAMPS `len(ctx)` u8 range)
and the Label (30 bytes) sits comfortably inside that.

**Key-derivation note.** LAMPS uses 32-byte seeds. `ml_dsa::SigningKey::<MlDsa65>::from_seed(&Seed)`
(`Seed = Array<u8,U32>`) derives the key deterministically; `verifying_key()` /
`expanded_key()` give the pieces. Benten's current `sig.rs` instead uses
`SigningKey::generate()` (random) — for the faithful impl, key handling is unchanged
in principle (still a 32-byte seed); only the *sign/verify call* changes to the
`ctx`-bearing form over `M'`.

> Version nuance: the scratch crate's fresh resolve picked `ml-dsa 0.1.1` (a patch
> bump); the workspace lockfile pins `0.1.0`. I read the `0.1.0` source directly and
> confirmed `verify_with_context` / `sign_deterministic` / `sign_randomized` /
> `expanded_key()` are all `pub` in `0.1.0` — the API used is identical across both
> patch versions. No version bump is required to do the faithful impl.

---

## PRODUCTION RE-IMPL SHAPE

The re-impl is a **construction swap inside the existing `0x0001` codepoint**, not a
new primitive. Concrete recipe:

**1. Message representative (new helper).**
```rust
// In benten-crypto-suite::sig (or a new lamps submodule)
const LAMPS_PREFIX: &[u8; 32] = b"CompositeAlgorithmSignatures2025";
const LAMPS_LABEL_MLDSA65_ED25519_SHA512: &[u8; 30] = b"COMPSIG-MLDSA65-Ed25519-SHA512";

fn lamps_m_prime(ctx: &[u8], msg: &[u8]) -> Vec<u8> {     // ctx.len() must be <= 255
    let ph = sha2::Sha512::digest(msg);                   // SHA-512(M), 64 B
    let mut m = Vec::with_capacity(32 + 30 + 1 + ctx.len() + 64);
    m.extend_from_slice(LAMPS_PREFIX);
    m.extend_from_slice(LAMPS_LABEL_MLDSA65_ED25519_SHA512);
    m.push(ctx.len() as u8);
    m.extend_from_slice(ctx);
    m.extend_from_slice(&ph);
    m
}
```

**2. Sign.**
```rust
let mp = lamps_m_prime(ctx, msg);
let mldsa_sig = kp.pq.expanded_key()
    .sign_randomized(&mp, LAMPS_LABEL_..., &mut OsRng)   // or sign_deterministic for det. mode
    .expect("ctx <= 255");
let trad_sig = kp.classical.sign(&mp);                   // Ed25519 over M', no ctx
// wire = mldsa_sig.encode()(3309) || trad_sig.to_bytes()(64)
```

**3. Verify.**
```rust
let mp = lamps_m_prime(ctx, msg);
let trad_ok = ed_vk.verify(&mp, &trad_sig).is_ok();
let mldsa_ok = ml_vk.verify_with_context(&mp, LAMPS_LABEL_..., &mldsa_sig);
// fail-closed if either is false (BOTH-must-verify is preserved)
```

**4. Wire/serialization changes (the breaking part).**
- Order flips: `mldsaSig(3309) || tradSig(64)` (ML-DSA FIRST) vs Benten today
  `classical(64) || pq(3309) || commitment(32)` (Ed25519 first + trailer).
- Offsets become `TRAD_SIG_OFFSET = 3309`, `TRAD_PK_OFFSET = 1952` (already pinned in
  `f_kat_4`). Composite pk = `mldsaPK(1952) || tradPK(32)`; total sig 3373 B, total pk 1984 B.
- A new `ctx` input threads into sign/verify (empty-ctx is the common case; the
  Varsig header / `SignatureSuite` surface must carry/derive it). For Benten's
  internal flows `ctx = b""` is the default; the empty-ctx vector is the one to pin
  green first.

**New public surface needed.**
- A LAMPS composite signer/verifier path on `SignatureSuite` (likely
  `sign_lamps(&kp, msg, ctx)` / `verify_lamps(...)` or a reshaped `sign`/`verify` that
  builds `M'`).
- Probably no new `ErrorCode` for the happy path (reuse `MalformedSignature` /
  fail-closed), but inbound cross-ecosystem acceptance (the still-`#[ignore]`'d
  `benten_accepts_cross_ecosystem_lamps_signatures`) becomes wire-able once this lands
  — it can then accept real BouncyCastle/OpenSSL/OpenPGP composites.
- The `f_kat_4` outbound-shape pin (`benten_lamps_signature_outbound_shape`) currently
  asserts `ed25519_half.len()==64 || mldsa65_half.len()==3309` from
  `classical_half_for_test`/`pq_half_for_test`; those helpers + the wire layout in
  `HybridSignature::to_wire_bytes` change order.

**Estimated risk / size.** **LOW-MEDIUM risk, ~250–450 LOC + test churn.** The crypto
is proven (this spike). It's mechanical: a `lamps_m_prime` helper, swap the two
`sign`/`verify` call sites to the `ctx`-bearing methods over `M'`, flip the wire
order, and decide the commitment-trailer disposition (below). The main blast radius is
**wire-format**: `to_wire_bytes` / Varsig round-trip / `from_parts_internal` /
`signature_byte_len_for(0x0001)` (3373 not 3405-with-commitment) / every golden-hex /
freeze byte / the cross-lang TS mirror. Because `0x0001` is a FROZEN v1-beta codepoint,
this is a **wire-format-affecting freeze-level change** — full-phase re-verify of every
golden + a Ben freeze-gate, not a quiet swap.

---

## GOTCHAS / CAVEATS

1. **`ml_dsa` version.** Pinned `0.1.0` has the full API (`verify_with_context`,
   `sign_deterministic`, `sign_randomized`, `expanded_key()`). No bump needed. The
   `sign`/`MultipartSigner` trait impls are EMPTY-CTX ONLY — do NOT use the bare
   `Signer::sign(msg)` for the faithful impl; it omits the Label ctx and signs the raw
   message. Use the `ExpandedSigningKey` ctx-bearing methods explicitly.

2. **NF-4 SHA3-256 commitment disposition — MUST DROP from the default wire.** The IETF
   LAMPS composite has NO slot for Benten's 32-byte `SHA3-256(domain||pubs||sigs)`
   commitment trailer. A byte-faithful `0x0001` therefore CANNOT carry it. Two options:
   (a) **DROP it** (recommended for byte-faithfulness) — strip-resistance then rests on
   the LAMPS shared-`M'`/`mldsa_ctx=Label` binding (the draft's own mechanism: both
   halves cover the same `M'`, and the ML-DSA half additionally binds the Label as
   ctx). This is the standard LAMPS security argument; it's what every other ecosystem
   relies on. (b) Retain the commitment ONLY on a **separate Benten-specific
   non-default codepoint** (a different SigCodepoint), never on `0x0001`. Note dropping
   the commitment removes a Benten-only belt-and-suspenders layer; the LAMPS analysis
   (EUF-CMA / Weakly-Non-Separable) is what remains — which is exactly what
   Compromise #31 already documents, so no *new* posture is introduced.

3. **EUF-CMA / WNS inheritance — UNCHANGED, inherits exactly Compromise #31.** A
   byte-faithful impl IS the LAMPS construction, so it inherits precisely the
   `draft-19 §9.2.2` (EUF-CMA-only; "NOT RECOMMENDED where EUF-CMA not shown
   acceptable") + `§10` (Weakly-Non-Separable, NOT Strongly-Non-Separable) analysis that
   `docs/SECURITY-POSTURE.md` Compromise #31 already cites verbatim, and the SUF-CMA
   gap stays closed at the app layer by Inv-15. The current NF-4 construction is already
   labelled "LAMPS Composite ML-DSA", so the security framing carries over with no
   change; the commitment-drop (gotcha 2) does not weaken the documented posture
   (Compromise #31 never credited the commitment as the SUF closure — Inv-15 is).
   **Action: Compromise #31 needs a one-line note that the construction is now the
   byte-faithful IETF wire (not just principle-aligned); the EUF/WNS claims stand.**

4. **draft-19 is NOT an RFC yet — stability caveat.** `id-MLDSA65-Ed25519-SHA512`, the
   OID `1.3.6.1.5.5.7.6.48` (IANA early-allocated 2025-10-20), the Prefix
   `"CompositeAlgorithmSignatures2025"`, and the Label are stable across recent drafts,
   but a draft can still change before RFC. The Prefix string literally embeds `2025`,
   and the test vectors were re-run 2026-01-07 — i.e. the construction is settling but
   not frozen by IETF. Benten's crypto-agility framework (baked-in #5) absorbs any
   pre-RFC change as an additive codepoint, but pinning `0x0001` to the draft-19 bytes
   accepts a (small) risk of a pre-RFC wire change. Recommend: pin to the exact
   draft-19 + the `f0627ab3` test-vector commit, document the draft-not-RFC status at
   the freeze, and re-verify against the final RFC when it lands.

5. **ctx threading is new surface.** Benten's flows are empty-ctx today; the faithful
   impl adds a `ctx` parameter end-to-end. Empty-ctx (`len=0x00`, no ctx bytes) is the
   default and the first thing to pin green (the `s` field of the vector). The
   non-empty-ctx path (`sWithContext`) also verified here, so the impl can support
   application contexts later without re-proving the crypto.

6. **Hedged vs deterministic.** The published vector's ML-DSA half is **hedged**
   (randomized). For Benten, hedged (`sign_randomized` + OsRng) is the FIPS-recommended
   default and matches the ecosystem; deterministic (`sign_deterministic`) is available
   if reproducible-signature behavior is ever wanted. Either interoperates. Note: if any
   Benten flow keys off signature-byte determinism it must NOT assume reproducibility —
   but Inv-15 already forbids keying off sig bytes, so this is a non-issue.

---

## BOTTOM LINE

The byte-faithful LAMPS re-impl is **DE-RISKED and GREEN-LIGHT-able**. The real
published vector verifies with Benten's own pinned primitives; the exact recipe is
confirmed (incl. the load-bearing `mldsa_ctx=Label`); and the pinned `ml-dsa 0.1.0`
already exposes the context-bearing sign/verify the construction requires — no version
bump, no upstream fork, no missing API. The work is a wire-format-affecting (freeze-
gated) construction swap of ~250–450 LOC, whose main cost is golden/freeze-byte churn
on the frozen `0x0001` codepoint and the NF-4-commitment-drop decision, NOT crypto
uncertainty.
