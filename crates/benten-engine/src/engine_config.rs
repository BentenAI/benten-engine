//! Workspace-level engine configuration loader (Phase 2b G7-A — Ben's
//! brief addition).
//!
//! Reads `engine.toml` from the workspace root at [`crate::Engine::open`]
//! time (or on demand via [`EngineConfig::load_or_default`]). The config
//! lets a deployment override engine-wide defaults without recompiling
//! — most importantly the D24 SANDBOX wallclock default + ceiling.
//!
//! ## Precedence
//!
//! ```text
//! per-handler `wallclock_ms` opt-in (SubgraphSpec.primitives, G12-D)
//!     └── overrides ──>
//! engine.toml `[sandbox] wallclock_default_ms` / `wallclock_max_ms`
//!     └── overrides ──>
//! built-in default (D24-RESOLVED: 30s default / 5min ceiling)
//! ```
//!
//! Per-handler clamps to engine.toml ceiling (the per-handler value
//! cannot exceed `wallclock_max_ms`). Engine.toml ceiling clamps to no
//! upper bound — operators who want to relax beyond the built-in 5min
//! can do so by setting a higher `wallclock_max_ms` value, accepting
//! the security tradeoff.
//!
//! ## File format
//!
//! ```toml
//! [sandbox]
//! # Optional. D24 default 30000 (30s). Used when handler does not
//! # specify per-handler wallclock_ms.
//! wallclock_default_ms = 60000
//!
//! # Optional. D24 default 300000 (5min). Per-handler values above this
//! # are clamped at validation time.
//! wallclock_max_ms = 1800000
//! ```
//!
//! Missing `engine.toml` ⇒ all built-in defaults apply.
//! Malformed `engine.toml` ⇒ [`ErrorCode::EngineConfigInvalid`].
//!
//! ## Compile-time wasm32 disable
//!
//! Mirrors the SANDBOX wasm32 cut (sec-pre-r1-05) — the loader uses
//! `toml` which builds for native; on wasm32 the engine config is
//! always built-in defaults.

#![cfg(not(target_arch = "wasm32"))]

use benten_errors::ErrorCode;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

// Re-export the D24 constants so `engine.toml` precedence chain has a
// single naming source.
pub use benten_eval::sandbox::{WALLCLOCK_DEFAULT_MS, WALLCLOCK_MAX_MS};

/// Workspace-level engine configuration. Loaded from `engine.toml` at
/// workspace root via [`Self::load_or_default`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct EngineConfig {
    /// SANDBOX subsection (D24 wallclock overrides etc.).
    #[serde(default)]
    pub sandbox: SandboxSection,
}

/// SANDBOX-section knobs.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct SandboxSection {
    /// Override the built-in D24 wallclock default (30s). When absent,
    /// the built-in applies.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub wallclock_default_ms: Option<u64>,
    /// Override the built-in D24 wallclock ceiling (5min). Per-handler
    /// values above this are clamped to typed-error
    /// `E_SANDBOX_WALLCLOCK_INVALID` at validation time.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub wallclock_max_ms: Option<u64>,
}

impl EngineConfig {
    /// Load `engine.toml` from the given workspace root. Missing file
    /// ⇒ `Ok(EngineConfig::default())`. Malformed file ⇒
    /// `Err(EngineConfigError::Parse { .. })` which routes to
    /// `E_ENGINE_CONFIG_INVALID`.
    ///
    /// # Errors
    /// Returns [`EngineConfigError::Parse`] when the file exists but is
    /// not valid TOML or fails the [`EngineConfig`] schema; returns
    /// [`EngineConfigError::Io`] when read fails for an unrelated I/O
    /// reason.
    pub fn load_or_default(workspace_root: &Path) -> Result<Self, EngineConfigError> {
        let path = workspace_root.join("engine.toml");
        match std::fs::read_to_string(&path) {
            Ok(text) => toml::from_str::<Self>(&text).map_err(|e| EngineConfigError::Parse {
                path: path.clone(),
                reason: e.to_string(),
            }),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Self::default()),
            Err(e) => Err(EngineConfigError::Io {
                path,
                reason: e.to_string(),
            }),
        }
    }

    /// Effective default wallclock — `engine.toml` override OR D24 built-in.
    #[must_use]
    pub fn effective_wallclock_default_ms(&self) -> u64 {
        self.sandbox
            .wallclock_default_ms
            .unwrap_or(WALLCLOCK_DEFAULT_MS)
    }

    /// Effective wallclock ceiling — `engine.toml` override OR D24 built-in.
    #[must_use]
    pub fn effective_wallclock_max_ms(&self) -> u64 {
        self.sandbox.wallclock_max_ms.unwrap_or(WALLCLOCK_MAX_MS)
    }

    /// Validate a per-handler `wallclock_ms` against the precedence
    /// chain (per-handler ≤ engine.toml ceiling). Returns
    /// `Err(ErrorCode::SandboxWallclockInvalid)` when violated.
    ///
    /// `None` per-handler ⇒ effective default used (no validation
    /// needed since the default is already trusted).
    ///
    /// # Errors
    /// Returns [`ErrorCode::SandboxWallclockInvalid`] when the
    /// per-handler value is `0` or exceeds the effective ceiling.
    pub fn validate_per_handler_wallclock(&self, ms: Option<u64>) -> Result<u64, ErrorCode> {
        match ms {
            None => Ok(self.effective_wallclock_default_ms()),
            Some(0) => Err(ErrorCode::SandboxWallclockInvalid),
            Some(v) if v > self.effective_wallclock_max_ms() => {
                Err(ErrorCode::SandboxWallclockInvalid)
            }
            Some(v) => Ok(v),
        }
    }
}

/// Failure modes for [`EngineConfig::load_or_default`].
#[derive(Debug, Clone, thiserror::Error, PartialEq, Eq)]
#[non_exhaustive]
pub enum EngineConfigError {
    /// `engine.toml` exists but failed to parse against the
    /// [`EngineConfig`] schema. Maps to `E_ENGINE_CONFIG_INVALID`.
    #[error("engine.toml at {} parse failure: {reason}", path.display())]
    Parse {
        /// Path to the offending file.
        path: PathBuf,
        /// Human-readable reason (raw `toml` parser message).
        reason: String,
    },
    /// `engine.toml` exists but read failed for an unrelated I/O
    /// reason (permissions, locked, etc.). Maps to
    /// `E_ENGINE_CONFIG_INVALID` (operators see a single error class
    /// for "your engine.toml is broken").
    #[error("engine.toml at {} read failure: {reason}", path.display())]
    Io {
        /// Path to the file.
        path: PathBuf,
        /// Human-readable reason.
        reason: String,
    },
}

impl EngineConfigError {
    /// Stable catalog code for routing.
    #[must_use]
    pub fn code(&self) -> ErrorCode {
        ErrorCode::EngineConfigInvalid
    }
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn missing_engine_toml_yields_defaults() {
        let dir = TempDir::new().expect("temp dir");
        let cfg = EngineConfig::load_or_default(dir.path()).expect("missing OK");
        assert_eq!(cfg.effective_wallclock_default_ms(), WALLCLOCK_DEFAULT_MS);
        assert_eq!(cfg.effective_wallclock_max_ms(), WALLCLOCK_MAX_MS);
    }

    #[test]
    fn engine_toml_overrides_wallclock_default() {
        let dir = TempDir::new().expect("temp dir");
        std::fs::write(
            dir.path().join("engine.toml"),
            "[sandbox]\nwallclock_default_ms = 60000\n",
        )
        .expect("write");
        let cfg = EngineConfig::load_or_default(dir.path()).expect("loads");
        assert_eq!(cfg.effective_wallclock_default_ms(), 60_000);
        // Ceiling untouched — built-in.
        assert_eq!(cfg.effective_wallclock_max_ms(), WALLCLOCK_MAX_MS);
    }

    #[test]
    fn engine_toml_overrides_wallclock_max() {
        let dir = TempDir::new().expect("temp dir");
        std::fs::write(
            dir.path().join("engine.toml"),
            "[sandbox]\nwallclock_max_ms = 1800000\n",
        )
        .expect("write");
        let cfg = EngineConfig::load_or_default(dir.path()).expect("loads");
        assert_eq!(cfg.effective_wallclock_max_ms(), 1_800_000);
    }

    #[test]
    fn malformed_engine_toml_routes_typed_error() {
        let dir = TempDir::new().expect("temp dir");
        std::fs::write(dir.path().join("engine.toml"), "this is not toml [[[").expect("write");
        let err = EngineConfig::load_or_default(dir.path()).unwrap_err();
        assert_eq!(err.code(), ErrorCode::EngineConfigInvalid);
    }

    #[test]
    fn validate_per_handler_wallclock_within_ceiling() {
        let cfg = EngineConfig::default();
        let v = cfg.validate_per_handler_wallclock(Some(60_000)).unwrap();
        assert_eq!(v, 60_000);
    }

    #[test]
    fn validate_per_handler_wallclock_above_ceiling_rejected() {
        let cfg = EngineConfig::default();
        let err = cfg
            .validate_per_handler_wallclock(Some(WALLCLOCK_MAX_MS + 1))
            .unwrap_err();
        assert_eq!(err, ErrorCode::SandboxWallclockInvalid);
    }

    #[test]
    fn validate_per_handler_wallclock_zero_rejected() {
        let cfg = EngineConfig::default();
        let err = cfg.validate_per_handler_wallclock(Some(0)).unwrap_err();
        assert_eq!(err, ErrorCode::SandboxWallclockInvalid);
    }

    #[test]
    fn validate_per_handler_wallclock_none_yields_effective_default() {
        let cfg = EngineConfig::default();
        let v = cfg.validate_per_handler_wallclock(None).unwrap();
        assert_eq!(v, WALLCLOCK_DEFAULT_MS);
    }

    #[test]
    fn engine_toml_relaxed_ceiling_lets_per_handler_above_d24_default() {
        // Test the "test environments" use case in Ben's brief addition.
        let dir = TempDir::new().expect("temp dir");
        std::fs::write(
            dir.path().join("engine.toml"),
            "[sandbox]\nwallclock_max_ms = 1800000\n",
        )
        .expect("write");
        let cfg = EngineConfig::load_or_default(dir.path()).expect("loads");
        // 10min is above D24 5min ceiling but below relaxed ceiling.
        let v = cfg.validate_per_handler_wallclock(Some(600_000)).unwrap();
        assert_eq!(v, 600_000);
    }
}
