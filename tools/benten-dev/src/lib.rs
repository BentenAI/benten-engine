//! # benten-dev
//!
//! Phase-2a G11-A dev-server stub. Owns hot-reload, inspect-state CLI, and
//! grant-preservation contracts. The bodies are `todo!()` — the R5 G11-A
//! group lands the real implementation. Tests reach these surfaces and fail
//! loudly with the R5 pointer, preserving the TDD red-phase contract.
//!
//! Traces to `.addl/phase-2a/00-implementation-plan.md` §3 G11-A +
//! `.addl/phase-2a/r2-test-landscape.md` §2.9.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use benten_core::{Cid, Value};
use benten_errors::ErrorCode;
use std::path::{Path, PathBuf};

/// Dev-server handle. Holds the workspace root + (in G11-A) the backing
/// engine, hot-reload coordinator, and grant table.
pub struct DevServer {
    _workspace: PathBuf,
}

/// Builder used by tests.
pub struct DevServerBuilder {
    workspace: Option<PathBuf>,
}

impl DevServer {
    /// Start a fresh builder.
    #[must_use]
    pub fn builder() -> DevServerBuilder {
        DevServerBuilder { workspace: None }
    }

    /// Grant a capability for the given actor. G11-A wires through the real
    /// capability backend.
    ///
    /// # Errors
    /// Returns `Err(ErrorCode)` when the backend refuses the grant.
    pub fn grant(&mut self, _actor: &Cid, _scope: &str) -> Result<(), ErrorCode> {
        todo!(
            "phase-2a-G11-A: DevServer::grant — see `.addl/phase-2a/00-implementation-plan.md` §3 G11-A"
        )
    }

    /// Whether the given actor currently holds the given scope.
    #[must_use]
    pub fn grant_exists(&self, _actor: &Cid, _scope: &str) -> bool {
        todo!("phase-2a-G11-A: DevServer::grant_exists — see plan §3 G11-A")
    }

    /// Testing shim — drives the attenuation walker.
    ///
    /// # Errors
    /// Returns `Err(ErrorCode)` when the policy denies.
    pub fn check_attenuation_for_test(&self, _actor: &Cid, _scope: &str) -> Result<(), ErrorCode> {
        todo!("phase-2a-G11-A: DevServer::check_attenuation_for_test — see plan §3 G11-A")
    }

    /// Register a handler from a DSL source string. G11-A fixes the spelling
    /// of the dev-server's text surface.
    ///
    /// # Errors
    /// Returns `Err(ErrorCode)` on parse / register failure.
    pub fn register_handler_from_str(
        &self,
        _handler_id: &str,
        _op: &str,
        _source: &str,
    ) -> Result<(), ErrorCode> {
        todo!("phase-2a-G11-A: DevServer::register_handler_from_str — see plan §3 G11-A")
    }

    /// Testing shim — triggers a hot-reload.
    ///
    /// # Errors
    /// Returns `Err(ErrorCode)` if the reload fails.
    pub fn reload_for_test(&self) -> Result<(), ErrorCode> {
        todo!("phase-2a-G11-A: DevServer::reload_for_test — see plan §3 G11-A")
    }

    /// Testing shim — audit sequence of the underlying grant table.
    #[must_use]
    pub fn grant_table_audit_sequence_for_test(&self) -> u64 {
        todo!("phase-2a-G11-A: DevServer::grant_table_audit_sequence_for_test — see plan §3 G11-A")
    }

    /// Explicit reset of dev state (clears grants; separate from hot-reload).
    ///
    /// # Errors
    /// Returns `Err(ErrorCode)` on reset failure.
    pub fn reset_dev_state(&mut self) -> Result<(), ErrorCode> {
        todo!("phase-2a-G11-A: DevServer::reset_dev_state — see plan §3 G11-A")
    }

    /// Testing shim — call a handler. Returns an outcome carrying the
    /// handler-version tag.
    ///
    /// # Errors
    /// Returns `Err(ErrorCode)` when the call fails.
    pub fn call_for_test(
        &self,
        _handler_id: &str,
        _op: &str,
        _input: Value,
    ) -> Result<DevCallOutcome, ErrorCode> {
        todo!("phase-2a-G11-A: DevServer::call_for_test — see plan §3 G11-A")
    }

    /// Testing shim — call with suspension and return raw bytes.
    ///
    /// # Errors
    /// Returns `Err(ErrorCode)` on failure.
    pub fn call_with_suspension_for_test(
        &self,
        _handler_id: &str,
        _op: &str,
        _input: Value,
    ) -> Result<Vec<u8>, ErrorCode> {
        todo!("phase-2a-G11-A: DevServer::call_with_suspension_for_test — see plan §3 G11-A")
    }

    /// Testing shim — resume a suspended call by bytes.
    ///
    /// # Errors
    /// Returns `Err(ErrorCode)` on failure.
    pub fn resume_for_test(
        &self,
        _bytes: &[u8],
        _signal: Value,
    ) -> Result<DevCallOutcome, ErrorCode> {
        todo!("phase-2a-G11-A: DevServer::resume_for_test — see plan §3 G11-A")
    }
}

impl DevServerBuilder {
    /// Set the workspace root.
    #[must_use]
    pub fn workspace(mut self, path: &Path) -> Self {
        self.workspace = Some(path.to_path_buf());
        self
    }

    /// Build a dev server. Stub — returns a shell that will panic with the
    /// R5 pointer on any method call.
    ///
    /// # Errors
    /// Returns `Err(ErrorCode)` when the workspace is missing.
    pub fn build(self) -> Result<DevServer, ErrorCode> {
        let workspace = self.workspace.unwrap_or_else(|| PathBuf::from("."));
        Ok(DevServer {
            _workspace: workspace,
        })
    }
}

/// Outcome surface for dev-server calls. Fields are `todo!()`-accessed.
#[derive(Debug)]
pub struct DevCallOutcome {
    _priv: (),
}

impl DevCallOutcome {
    /// Testing shim — the version tag of the subgraph this outcome was
    /// produced from.
    #[must_use]
    pub fn handler_version_tag_for_test(&self) -> &'static str {
        todo!("phase-2a-G11-A: DevCallOutcome::handler_version_tag_for_test — see plan §3 G11-A")
    }
}
