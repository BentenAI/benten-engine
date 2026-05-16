//! Safe-1 #512 — `RevocationRegistry` mutex-poison would-FAIL pin.
//!
//! pim-2 / §3.6b end-to-end closure pin for the mutex-poison
//! hardening landed in the #1175 bundle (`fix(benten-id): Hyg-1
//! dead-code sweep + mutex-poison hardening`). The fix replaced
//! `.expect("registry mutex poisoned")` with
//! `.unwrap_or_else(std::sync::PoisonError::into_inner)` on both
//! `RevocationRegistry::revoke` and `::is_revoked`.
//!
//! This test deliberately poisons the registry's internal `Mutex`
//! (a spawned thread panics while holding the lock — the same poison
//! path a panicking real caller would trigger) then asserts the
//! post-fix recovery path still produces a CORRECT, fail-CLOSED
//! revocation answer.
//!
//! **Would-FAIL-if-reverted proof:** with the pre-fix
//! `.expect("registry mutex poisoned")` body, both `revoke()` and
//! `is_revoked()` PANIC on the poisoned lock — the test panics
//! (process-level for `is_revoked`'s assertion arm) instead of
//! returning `true`, so the assertion below is unreachable and the
//! test fails. The test exercises the real recovered production
//! path; it is not a sentinel-presence check.

#![allow(clippy::unwrap_used)]

use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::Arc;
use std::thread;

use benten_id::vc::RevocationRegistry;

#[test]
fn revocation_registry_recovers_from_poisoned_mutex_and_stays_fail_closed() {
    let registry = Arc::new(RevocationRegistry::new());

    // Seed a known revocation BEFORE poisoning so we can prove the
    // recovered guard still observes prior state (poison must not
    // lose the HashSet contents — `into_inner` recovery preserves it).
    registry.revoke("urn:status:pre-poison");

    // Poison the internal mutex: a thread panics while holding the
    // lock. We reach into the lock via a `revoke` call wrapped so the
    // panic happens *inside* the critical section. The cleanest way
    // to guarantee the panic lands while the guard is held is to
    // panic from a closure the registry runs under the lock — but the
    // public API doesn't expose that, so we poison via a panicking
    // thread that holds a guard transitively through `revoke`'s
    // internal lock by panicking right after re-entrancy is not
    // possible. Instead: spawn a thread that takes the lock through a
    // helper that panics. Since `Mutex` poisons on ANY panic while a
    // guard from it is alive, we trigger that by having the thread
    // acquire the guard (via `revoke`, which holds it briefly) is not
    // enough — we need the panic WHILE held. Use a second registry
    // handle and a panic injected through `is_revoked` is also not
    // held-across-panic. The robust route: poison through a manual
    // guard is impossible (field is private), so we induce poison via
    // a panicking destructor running while the lock is held.
    //
    // Practical approach that works against the real public surface:
    // spawn a thread that calls `revoke` with a key whose `Into<String>`
    // conversion panics — the conversion is evaluated by `revoke`
    // AFTER `.lock()` succeeds and the guard is live, so the panic
    // unwinds with the guard held → mutex poisoned.
    let reg = Arc::clone(&registry);
    let handle = thread::spawn(move || {
        reg.revoke(PanicOnIntoString);
    });
    let join = handle.join();
    assert!(
        join.is_err(),
        "the poisoning thread must have panicked (lock held across panic)"
    );

    // Post-fix: the recovered guard path must still work.
    //
    // 1. Prior state survives poison recovery (fail-CLOSED: a
    //    previously-revoked credential is STILL observed revoked).
    let still_revoked = catch_unwind(AssertUnwindSafe(|| {
        registry.is_revoked("urn:status:pre-poison")
    }));
    assert_eq!(
        still_revoked.ok(),
        Some(true),
        "is_revoked must NOT panic on the poisoned lock and must \
         still report the pre-poison revocation (fail-CLOSED). \
         With the pre-fix `.expect()` this panics → test fails."
    );

    // 2. New revocations still take effect through the recovered
    //    guard (the registry remains usable after poison).
    let revoke_ok = catch_unwind(AssertUnwindSafe(|| {
        registry.revoke("urn:status:post-poison");
    }));
    assert!(
        revoke_ok.is_ok(),
        "revoke must NOT panic on the poisoned lock (PoisonError::into_inner \
         recovery). With the pre-fix `.expect()` this panics → test fails."
    );
    assert!(
        registry.is_revoked("urn:status:post-poison"),
        "post-poison revocation must be observable through the recovered guard"
    );
}

/// A type whose `Into<String>` conversion panics. `RevocationRegistry::revoke`
/// takes `impl Into<String>` and evaluates the conversion AFTER it has
/// acquired the mutex guard, so the panic unwinds with the guard live
/// → the mutex is poisoned exactly as a real panicking caller would.
struct PanicOnIntoString;

impl From<PanicOnIntoString> for String {
    fn from(_: PanicOnIntoString) -> String {
        panic!("intentional panic inside revoke() critical section to poison the mutex");
    }
}
