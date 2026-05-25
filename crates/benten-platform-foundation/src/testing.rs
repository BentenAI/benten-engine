//! R6 R1 FP-F4 §S2 — test-only helpers for the install pipeline ports.
//!
//! Gated behind the `testing` feature. Test fixtures use these in place
//! of the production wirings to keep per-test boilerplate minimal while
//! documenting which axes the test is NOT exercising.
//!
//! Production callers MUST NOT use these helpers — the §4.37 TOCTOU
//! replay defense relies on every install path threading the engine's
//! `install_record_replay_store().record_and_check` closure; the noop
//! helper here observably admits ALL install records as fresh.

use benten_errors::ErrorCode;

/// **R6 R1 FP-F4 §S2** — test-only no-op replay-check closure.
///
/// Returns an admit-all closure of the type
/// `InstallPorts::install_record_replay_check` requires. Test fixtures
/// that do NOT exercise the §4.37 TOCTOU replay-defense surface wire
/// this; tests that DO exercise replay-defense should wire a real
/// `Engine::install_record_replay_store().record_and_check` closure.
///
/// # Why this exists
///
/// Per Δv3-9 the prior `Option<>` wrapper on
/// `InstallPorts.install_record_replay_check` silently disabled the
/// §4.37 defense at every shipped binary; dropping the Option forces
/// every caller to make an explicit choice between substantive
/// defense (production) and explicit no-op (tests that aren't
/// exercising replay). This helper makes the test side trivially
/// callable while documenting the intent.
///
/// # Usage
///
/// ```ignore
/// let mut noop = benten_platform_foundation::testing::noop_replay_check();
/// let mut ports = InstallPorts {
///     cap_minter: &mut cap_minter,
///     private_ns: &mut private_ns,
///     install_record_replay_check: &mut noop,
/// };
/// ```
pub fn noop_replay_check() -> impl FnMut(&[u8; 32]) -> Result<(), ErrorCode> {
    |_hash: &[u8; 32]| -> Result<(), ErrorCode> { Ok(()) }
}
