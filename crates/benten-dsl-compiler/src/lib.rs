//! `benten-dsl-compiler` — Phase-2b DSL-text → `SubgraphSpec` compiler.
//!
//! **MINIMAL-FOR-DEVSERVER scope** per `r1-architect-reviewer.json` (G12-B-scope):
//! ~200-300 LOC, exactly 4 public items intended for `tools/benten-dev`:
//!
//! 1. [`compile_str`] — compile a DSL source string into a `SubgraphSpec`.
//! 2. [`compile_file`] — compile a DSL source file path into a `SubgraphSpec`.
//! 3. [`CompileError`] — typed compile error enum.
//! 4. [`Diagnostic`] — diagnostic shape devserver renders.
//!
//! Everything else is `pub(crate)` (or `#[doc(hidden)]` if it must be `pub`).
//! Surface stability is intentionally narrow so `cargo-public-api` baseline
//! locked at G6 first push does not freeze design space we have not earned the
//! right to freeze.
//!
//! ## Dep direction (arch-pre-r1-3 + plan §3.2 G12-B)
//!
//! - Depends on: `benten-core` (for `Subgraph` / `SubgraphSpec` types).
//! - **Must not** depend on `benten-eval` or `benten-graph` — preserves arch-1.
//! - Pinned at compile time by `crates/benten-dsl-compiler/tests/dep_direction.rs`
//!   and `crates/benten-engine/tests/no_dsl_compiler_dep.rs`.

#![allow(clippy::needless_pass_by_value)]
#![allow(dead_code)]

use std::path::Path;

use thiserror::Error;

// Re-export the SubgraphSpec the compiler emits. The actual type lives in
// benten-core post-G12-C migration; until G12-C lands, the import is gated
// behind `phase_2b_landed`. Until then, expose a placeholder type alias so
// the public surface compiles in default builds.
//
// G12-B R5 implementer flips this to a real `pub use benten_core::SubgraphSpec;`
// once G12-D widens the spec.
#[doc(hidden)]
pub type SubgraphSpec = (); // placeholder — replaced in R5 G12-B green-phase

/// Compile a DSL source string into a [`SubgraphSpec`].
///
/// # Errors
///
/// Returns [`CompileError`] for any parse, type-check, or emission failure.
/// Each error carries a [`Diagnostic`] with source span + human-readable
/// message + typed `error_code` for devserver rendering.
pub fn compile_str(_source: &str) -> Result<SubgraphSpec, CompileError> {
    todo!("R5 G12-B implements DSL parser + emitter")
}

/// Compile a DSL source file into a [`SubgraphSpec`].
///
/// # Errors
///
/// Returns [`CompileError`] for IO failures or any failure modes of
/// [`compile_str`].
pub fn compile_file(_path: &Path) -> Result<SubgraphSpec, CompileError> {
    todo!("R5 G12-B implements file-based compile entry point")
}

/// Typed compile-error enum surfaced to devserver + downstream tools.
///
/// Wire-stable variant set: each variant maps to a stable `error_code` string
/// (see [`Diagnostic::error_code`]) so devserver / TS-side renderers can switch
/// on the discriminant without prose-string parsing.
#[derive(Debug, Error)]
pub enum CompileError {
    /// Lexer / parser failure — DSL did not match the grammar.
    #[error("DSL parse error: {0}")]
    Parse(Diagnostic),
    /// Semantic / type-check failure — DSL parsed but referenced unknown
    /// primitives / props / handler ids.
    #[error("DSL semantic error: {0}")]
    Semantic(Diagnostic),
    /// Emission failure — well-typed AST but `SubgraphSpec` construction
    /// rejected (e.g. exceeds primitive count cap, missing RESPOND, etc.).
    #[error("DSL emit error: {0}")]
    Emit(Diagnostic),
    /// IO failure reading a source file (only from [`compile_file`]).
    #[error("DSL io error: {0}")]
    Io(String),
}

/// Diagnostic shape devserver renders: span + message + typed error code.
#[derive(Debug, Clone)]
pub struct Diagnostic {
    /// Stable error-code string (e.g. `"E_DSL_PARSE_ERROR"`,
    /// `"E_DSL_UNKNOWN_PRIMITIVE"`); switch-keyed by devserver renderer.
    pub error_code: &'static str,
    /// Human-readable message for tooltip + log surface.
    pub message: String,
    /// 1-indexed line of the offending source span (None if span unknown).
    pub line: Option<u32>,
    /// 1-indexed column of the offending source span (None if span unknown).
    pub column: Option<u32>,
}

impl std::fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match (self.line, self.column) {
            (Some(l), Some(c)) => {
                write!(f, "[{}] {}:{} {}", self.error_code, l, c, self.message)
            }
            _ => write!(f, "[{}] {}", self.error_code, self.message),
        }
    }
}
