//! R3-A RED-PHASE pins for `benten-id` Ed25519 keypair (G14-A1, wave-4a).
//!
//! Pin sources (per `.addl/phase-3/r2-test-landscape.md` §2.2 G14-A1 +
//! `.addl/phase-3/00-implementation-plan.md` §3 G14-A1 must-pass column):
//!
//! - `tests/ed25519_keypair_round_trip` — plan §3 G14-A1
//! - `tests/keypair_secret_bytes_zeroized_on_drop` — `crypto-blocker-1` BLOCKER
//! - `tests/keypair_secret_does_not_implement_clone` — `crypto-blocker-1`
//! - `tests/keypair_secret_redacted_from_debug_display` — `crypto-blocker-1`
//! - `tests/keypair_clone_does_not_widen_lifetime` — `crypto-blocker-1`
//! - `tests/keypair_generate_uses_os_csprng` — `crypto-major-2`
//!
//! ## RED-PHASE discipline
//!
//! Every test in this file is `#[ignore]`'d with rationale
//! `"RED-PHASE: G14-A1 wave-4a fills benten-id::keypair"` because the
//! cited types (`benten_id::keypair::Keypair`, `SecretKey`, `PublicKey`)
//! don't exist yet (the crate is the wave-1pre stub landed by R3-A).
//! Per `feedback_end_to_end_test_pin_for_closed_claims` (§3.6b pim-2),
//! once G14-A1 lands the implementer:
//!
//! 1. Drops the `#[ignore]` attribute on each test.
//! 2. Wires the test against the real `benten_id::keypair` API.
//! 3. Verifies each test asserts an OBSERVABLE consequence (not just
//!    sentinel-presence): drop runs zeroize → memory inspection asserts
//!    bytes are zero; `Clone` not implemented → `Keypair::clone()` is a
//!    compile error in a `compile_fail` doctest; Debug/Display impl
//!    redacts → `format!("{kp:?}")` does not contain the secret bytes.
//!
//! **NOTE on test bodies:** Until `benten_id` exposes the real types,
//! the test bodies below are STRUCTURAL placeholders that document the
//! intended assertion shape. The implementer at G14-A1 replaces the
//! `unimplemented!()` body with the real assertion against the live API.
//! The `#[ignore]` rationale is the RED-PHASE pin source; un-ignoring
//! at G14-A1 is the lit-up signal that the surface has landed.

#![allow(clippy::unwrap_used)]

#[test]
#[ignore = "RED-PHASE: G14-A1 wave-4a fills benten-id::keypair (round-trip)"]
fn ed25519_keypair_round_trip() {
    // G14-A1 implementer wires this against the real API:
    //   let kp = benten_id::keypair::Keypair::generate();
    //   let pk = kp.public_key();
    //   let msg = b"hello";
    //   let sig = kp.sign(msg);
    //   assert!(pk.verify(msg, &sig).is_ok());
    //
    // Asserts the FULL Ed25519 contract end-to-end (generate → sign →
    // verify). Sentinel-presence (kp.is_some()) does not suffice per
    // §3.6b pim-2 end-to-end pin requirement.
    unimplemented!("G14-A1 wires Keypair::generate() + sign() + verify() round-trip");
}

#[test]
#[ignore = "RED-PHASE: G14-A1 — crypto-blocker-1 — secret zeroized on drop"]
fn keypair_secret_bytes_zeroized_on_drop() {
    // BLOCKER pin per crypto-blocker-1. G14-A1 implementer wires this
    // against the real API + a memory-inspection technique. Concrete
    // shape:
    //
    //   let secret_ptr_address = {
    //       let kp = benten_id::keypair::Keypair::generate();
    //       // SAFETY: addr-of for inspection only, no ref-aliasing.
    //       std::ptr::addr_of!(*kp.secret_bytes_for_test()) as usize
    //   }; // <-- kp dropped here; ZeroizeOnDrop runs
    //   // Re-read the same memory location and assert all zeros.
    //   let bytes_after_drop: [u8; 32] = unsafe {
    //       std::ptr::read_volatile(secret_ptr_address as *const [u8; 32])
    //   };
    //   assert_eq!(bytes_after_drop, [0u8; 32],
    //       "SecretKey::Drop must zeroize via `zeroize::ZeroizeOnDrop`");
    //
    // OBSERVABLE consequence: post-drop memory inspection sees zeros,
    // not the original 32-byte secret. This is the load-bearing
    // BLOCKER assertion per crypto-blocker-1.
    unimplemented!("G14-A1 wires zeroize::ZeroizeOnDrop assertion via memory inspection");
}

#[test]
#[ignore = "RED-PHASE: G14-A1 — crypto-blocker-1 — SecretKey: !Clone"]
fn keypair_secret_does_not_implement_clone() {
    // BLOCKER pin per crypto-blocker-1. The intent: the SECRET KEY
    // type MUST NOT implement `Clone` so secret bytes cannot be
    // duplicated outside the original lifetime (which Drop zeroizes).
    //
    // G14-A1 implementer wires this as a `trybuild`-style compile-fail
    // pin OR a static assertion. Static-assertion shape:
    //
    //   fn assert_not_clone<T>() where T: ?Sized {
    //       // Compiles iff T does NOT impl Clone. We use a sealed
    //       // trait with a default impl + a specialization opt-out.
    //       trait NotClone {} impl<T: ?Sized> NotClone for T {}
    //       trait IsClone { fn is_clone() -> bool { false } }
    //       impl<T: Clone> IsClone for T { fn is_clone() -> bool { true } }
    //       // Build error if T: Clone via the specialized impl picking
    //       // up `is_clone() -> true`, which the assertion below
    //       // contradicts.
    //   }
    //   assert_not_clone::<benten_id::keypair::SecretKey>();
    //
    // OBSERVABLE consequence: `let s2 = secret.clone();` fails to
    // compile. crypto-blocker-1 asserts this is the load-bearing
    // contract for secret-key handling discipline.
    unimplemented!("G14-A1 wires compile-fail or static-assert that SecretKey: !Clone");
}

#[test]
#[ignore = "RED-PHASE: G14-A1 — crypto-blocker-1 — Debug/Display redacts secret"]
fn keypair_secret_redacted_from_debug_display() {
    // BLOCKER pin per crypto-blocker-1. G14-A1 implementer wires this:
    //
    //   let kp = benten_id::keypair::Keypair::generate();
    //   let dbg = format!("{:?}", kp);
    //   let disp = format!("{}", kp);  // if Display impl present
    //   // The full hex-encoded secret (or any 32 contiguous secret bytes)
    //   // MUST NOT appear in the formatted output.
    //   let secret_hex = hex::encode(kp.secret_bytes_for_test());
    //   assert!(!dbg.contains(&secret_hex),
    //       "Debug impl must redact secret per crypto-blocker-1");
    //   assert!(dbg.contains("<redacted>") || dbg.contains("***"),
    //       "Debug impl should mark the redaction explicitly");
    //
    // OBSERVABLE consequence: tracing logs / panic prints / debug
    // dumps cannot accidentally leak the secret bytes. Defends
    // against the attack class where a stack-trace / structured-log
    // serializer reaches Debug on a Keypair held in scope.
    unimplemented!(
        "G14-A1 wires assertion that format!(\"{{:?}}\", kp) does not contain secret bytes"
    );
}

#[test]
#[ignore = "RED-PHASE: G14-A1 — crypto-blocker-1 — Clone does not widen lifetime"]
fn keypair_clone_does_not_widen_lifetime() {
    // crypto-blocker-1 pin. The intent: even if ANY clone-shape
    // operation exists on the public API (e.g., `Keypair::public_key()`
    // returns an owned `PublicKey`), it MUST NOT widen the secret's
    // lifetime. This test pins that calls returning derived material
    // (PublicKey, did:key DID) carry their own owned bytes — they do
    // not borrow from or reference-count the secret.
    //
    // G14-A1 implementer wires this:
    //
    //   let pk_owned = {
    //       let kp = benten_id::keypair::Keypair::generate();
    //       kp.public_key()  // owned PublicKey
    //   }; // kp dropped + zeroized
    //   // pk_owned is still usable — does not borrow from the dropped kp
    //   let did = pk_owned.to_did_key();
    //   assert!(did.as_str().starts_with("did:key:z"));
    //
    // OBSERVABLE consequence: the public-key path remains usable
    // after the secret is dropped/zeroized. If `public_key()` had
    // returned `&[u8]` borrowed from the secret, this test would
    // fail to compile.
    unimplemented!(
        "G14-A1 wires assertion that PublicKey outlives Keypair without lifetime widening"
    );
}

#[test]
#[ignore = "RED-PHASE: G14-A1 — crypto-major-2 — generate uses OS CSPRNG"]
fn keypair_generate_uses_os_csprng() {
    // crypto-major-2 pin. The intent: `Keypair::generate()` MUST be
    // pinned to the OS CSPRNG (via `getrandom` / `rand_core::OsRng`),
    // never a deterministic seed (which would generate identical
    // keypairs on every cold start, an authentication catastrophe).
    //
    // G14-A1 implementer wires this:
    //
    //   let kp1 = benten_id::keypair::Keypair::generate();
    //   let kp2 = benten_id::keypair::Keypair::generate();
    //   let kp3 = benten_id::keypair::Keypair::generate();
    //   assert_ne!(kp1.public_key().bytes(), kp2.public_key().bytes());
    //   assert_ne!(kp2.public_key().bytes(), kp3.public_key().bytes());
    //   assert_ne!(kp1.public_key().bytes(), kp3.public_key().bytes());
    //
    // The proptest at `prop_keypair_generate_distinct_across_1k_calls`
    // (sibling test file) is the load-bearing 10k-case verification;
    // this test is the smoke that 3 distinct public keys appear from 3
    // generate() calls — fast-fail signal vs the proptest's full
    // distribution check.
    unimplemented!(
        "G14-A1 wires assertion that 3 distinct generate() calls yield 3 distinct public keys"
    );
}
