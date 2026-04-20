//! Ergonomic `.lock_recover()` extension for `std::sync::Mutex` / `RwLock`.
//!
//! The crates sprinkle `lock().unwrap_or_else(|e| e.into_inner())` across
//! every Mutex guard acquisition because our recovery stance is "lock
//! poisoning is not a panic path — keep going with the poisoned state." That
//! idiom duplicated 35 times across 6 files (r6-ref R-major-03); this trait
//! collapses it to `.lock_recover()`.
//!
//! Re-exported from `benten_graph::MutexExt` / `benten_graph::RwLockExt` so
//! higher crates can use it without needing their own copy.

use std::sync::{Mutex, MutexGuard, RwLock, RwLockReadGuard, RwLockWriteGuard};

/// Extension on `std::sync::Mutex<T>` that collapses the poisoned-mutex
/// recovery idiom.
///
/// The single method `lock_recover()` behaves identically to
/// `lock().unwrap_or_else(|e| e.into_inner())`: if the mutex is poisoned we
/// still return the inner guard, because in this codebase poisoning means
/// "a previous holder panicked mid-critical-section" and the invariants
/// every critical section establishes are defensive enough that "keep
/// going" is the correct recovery.
pub trait MutexExt<T> {
    /// Acquire the mutex, recovering from poisoning by returning the inner
    /// guard. Never panics; blocks until the lock is available.
    fn lock_recover(&self) -> MutexGuard<'_, T>;
}

impl<T> MutexExt<T> for Mutex<T> {
    fn lock_recover(&self) -> MutexGuard<'_, T> {
        self.lock().unwrap_or_else(|e| e.into_inner())
    }
}

/// Extension on `std::sync::RwLock<T>` mirroring [`MutexExt`] for the
/// read/write-lock variants. Symmetric with `MutexExt::lock_recover`: a
/// poisoned lock yields the inner guard rather than propagating.
pub trait RwLockExt<T> {
    /// Acquire the RwLock for reading, recovering from poisoning.
    fn read_recover(&self) -> RwLockReadGuard<'_, T>;

    /// Acquire the RwLock for writing, recovering from poisoning.
    fn write_recover(&self) -> RwLockWriteGuard<'_, T>;
}

impl<T> RwLockExt<T> for RwLock<T> {
    fn read_recover(&self) -> RwLockReadGuard<'_, T> {
        self.read().unwrap_or_else(|e| e.into_inner())
    }

    fn write_recover(&self) -> RwLockWriteGuard<'_, T> {
        self.write().unwrap_or_else(|e| e.into_inner())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::thread;

    #[test]
    fn lock_recover_returns_guard_on_healthy_mutex() {
        let m = Mutex::new(42);
        assert_eq!(*m.lock_recover(), 42);
    }

    #[test]
    fn lock_recover_returns_guard_on_poisoned_mutex() {
        let m = Arc::new(Mutex::new(42));
        let m_cloned = Arc::clone(&m);
        let _ = thread::spawn(move || {
            let _guard = m_cloned.lock().unwrap();
            panic!("poison the mutex");
        })
        .join();
        // Poisoned, but lock_recover still yields the inner guard.
        assert_eq!(*m.lock_recover(), 42);
    }

    #[test]
    fn rwlock_recover_read_and_write() {
        let l = RwLock::new(7);
        assert_eq!(*l.read_recover(), 7);
        *l.write_recover() = 9;
        assert_eq!(*l.read_recover(), 9);
    }
}
