//! `benten-dsl-compiler` — Phase-2b DSL-text → `Subgraph` compiler.
//!
//! **MINIMAL-FOR-DEVSERVER scope** per `r1-architect-reviewer.json` (G12-B-scope):
//! ~900 LOC, 9 public items intended for `tools/benten-dev` (the literal LOC
//! ceiling drifted from the original ~200-400 framing as the parser + the
//! Phase-3 R6 `validate_shapes` pass + the 12-primitive dispatch landed; the
//! spirit-of-the-rule — one source file, narrow surface, no engine/eval/graph
//! deps — holds):
//!
//! 1. [`compile_str`] — compile a DSL source string into a [`CompiledSubgraph`].
//! 2. [`compile_file`] — compile a DSL source file path into a [`CompiledSubgraph`].
//! 3. [`CompileError`] — typed compile error enum.
//! 4. [`Diagnostic`] — diagnostic shape devserver renders.
//! 5. [`CompiledSubgraph`] — canonical [`Subgraph`] + per-primitive list.
//! 6. [`CompiledPrimitive`] — one primitive declaration for introspection.
//! 7. [`PrimitiveKind`] — re-export so consumers need no transitive `benten-core` dep.
//! 8. [`Span`] — half-open `[start_offset, end_offset)` byte-span carrying
//!    `(start_line, start_column, end_line, end_column)` for [`Diagnostic`]
//!    (G-CORE-DSL chunk-3, #1000 closure).
//! 9. [`CompileError::error_code`] — boundary helper resolving each variant to
//!    its stable `E_DSL_*` code string (G-CORE-DSL chunk-3, #839 closure).
//!
//! In addition, the narrow surface includes a small set of stable `E_DSL_*`
//! code-string `pub const`s — [`MAX_SOURCE_LEN`], [`E_DSL_PARSE_ERROR`],
//! [`E_DSL_UNKNOWN_PRIMITIVE`], [`E_DSL_MISSING_RESPOND`],
//! [`E_DSL_INVALID_SHAPE`], [`E_DSL_IO_ERROR`], [`E_DSL_BACKEND_REJECTED`] —
//! that mirror the [`benten_errors::ErrorCode`] catalog at this crate's
//! boundary (the §3.5g cross-language atomic-mirror seam).
//!
//! Everything else is `pub(crate)`. Surface stability is intentionally narrow
//! so `cargo-public-api` baseline locked at G6 first push does not freeze
//! design space we have not earned the right to freeze.
//!
//! ## Dep direction (arch-pre-r1-3 + plan §3.2 G12-B)
//!
//! - Depends on: `benten-core` (for `Subgraph` / `Value` / `PrimitiveKind`).
//! - **Must not** depend on `benten-eval`, `benten-graph`, or `benten-engine` —
//!   preserves arch-1.
//! - Pinned at test time by `tests/arch_n_benten_dsl_compiler_dep_direction.rs`
//!   (the four `#[test]` fns scan `Cargo.toml` source text on every
//!   `cargo test` / `cargo nextest run` + CI run — a forbidden dep would
//!   still compile but trips the next test invocation).
//!
//! ## Grammar (MINIMAL — Phase-2b devserver round-trip target)
//!
//! ```text
//! handler ::= 'handler' STRING '{' chain '}'
//! chain   ::= primitive ( '->' primitive )*
//! primitive ::=
//!   | 'read'      '(' STRING ')'
//!   | 'write'     '(' STRING ( ',' object )? ')'
//!   | 'transform' '(' object ')'
//!   | 'branch'    '(' expr ')'
//!   | 'wait'      '(' object ')'
//!   | 'call'      '(' STRING ( ',' object )? ')'
//!   | 'sandbox'   '(' STRING ( ',' object )? ')'
//!   | 'respond'
//! object  ::= '{' ( pair ( ',' pair )* )? '}'
//! pair    ::= IDENT ':' value
//! value   ::= STRING | NUMBER | BOOL | VAR | object
//! VAR     ::= '$' IDENT ( '.' IDENT )*
//! expr    ::= /* opaque text up to the matching ')'; stored as a Text Value */
//! STRING  ::= "'" [^']* "'"
//! ```
//!
//! The expression body of `branch(...)` is captured as an opaque text token
//! (the surface evaluator pins predicate semantics in a later phase). This
//! keeps the parser dead-simple while still satisfying the round-trip
//! property.
//!
//! ## Deliberate non-extensibility (no composability surfaces)
//!
//! This crate ships **zero** extension hooks by design — there is no
//! `PrimitiveParser` trait, no `PropertyHandler` registrar, no custom-
//! primitive-shorthand callback, no rule-registration surface on the
//! `validate_shapes` pass. The 12-primitive dispatch in `parse_primitive`
//! is a single hardcoded `match` with no extension arm. **This is
//! intentional, not an oversight:**
//!
//! - **CLAUDE.md #1** — the 12 operation primitives are irreducible.
//!   Extending the primitive set is rejected unless commitment #1 is
//!   re-opened; there is therefore deliberately no runtime hook to add a
//!   13th.
//! - **CLAUDE.md #19** — engine-level extensions are Rust crates compiled
//!   in, trusted because you compiled them. The DSL compiler is one such
//!   crate: a new primitive keyword or property rule is added by editing
//!   *this crate's source* (a reviewed `cargo` change), never by
//!   registering a runtime plugin.
//! - **CLAUDE.md #10** — the user-facing composability surface is the
//!   TypeScript DSL (`crud('post')` zero-config). The Rust-side DSL exists
//!   only for devserver inline compilation, not for end-user authoring.
//!
//! A reader asking "where do I register a custom primitive / property
//! handler?" — the answer is "you don't; edit this crate." See
//! `INTERNALS.md` §7 (MINIMAL-FOR-DEVSERVER scope) + §8 for the only
//! sanctioned future-extension path (schema-driven-rendering option (c)).

#![allow(clippy::needless_pass_by_value)]

use std::path::Path;

use benten_core::{Subgraph, Value};
use thiserror::Error;

// Public re-exports so devserver consumers never need to add a transitive
// `benten-core` dep just to read what the compiler produced.
pub use benten_core::PrimitiveKind;

// ---------------------------------------------------------------------------
// #604 / #782 — canonical property-key namespace (scheme-(a))
// ---------------------------------------------------------------------------
//
// Pre-v1 canonical-bytes normalization. Before this, the DSL compiler emitted
// underscore-prefixed property keys (`_target`, `_module`, `_body`, …) that
// DIVERGED from the keys `benten_core::SubgraphBuilder` stamps for the same
// `PrimitiveKind` (`handler`, `module`, `max`, …). The canonical bytes are a
// function of these keys (DAG-CBOR over the sorted `properties` BTreeMap), so
// the same logical handler authored via the Rust DSL vs the builder produced
// DIFFERENT CIDs — an Inv-10 cross-surface gap (#604) and an in-crate
// `_body`-overload (#782, Map for transform vs Text for iterate).
//
// Scheme-(a): the DSL conforms to the `SubgraphBuilder` canonical key names
// so both authoring surfaces yield byte-identical canonical encodings. These
// constants are the single source of truth; they mirror the literal keys in
// `benten_core::subgraph::SubgraphBuilder` (`call_handler` → "handler",
// `sandbox` → "module", `iterate` → "max", `transform` → "body", WAIT
// `wait_signal`/`wait_duration` → "signal"/"duration_ms"). The cross-doc
// type/name mirror discipline (dispatch-conventions §3.5g) couples these to
// the builder definitions.
//
// MUST land pre-v1 — CID churn of every DSL-authored handler is free now,
// catastrophic after the v1 wire-format freeze.

/// Canonical label key (READ/WRITE/STREAM target). Mirrors the structural
/// label semantics of `SubgraphBuilder::read`/`write` (no underscore).
const KEY_LABEL: &str = "label";
/// Canonical user-properties bag key (WRITE). Mirrors `SubgraphBuilder`
/// WRITE user-properties (no underscore).
const KEY_USER_PROPERTIES: &str = "user_properties";
/// Canonical TRANSFORM body key (Map). Mirrors `SubgraphBuilder::transform`.
const KEY_TRANSFORM_BODY: &str = "body";
/// Canonical BRANCH predicate key (Text).
const KEY_PREDICATE: &str = "predicate";
/// Canonical CALL target key. Mirrors `SubgraphBuilder::call_handler`'s
/// `"handler"` property.
const KEY_CALL_HANDLER: &str = "handler";
/// Canonical CALL args bag key.
const KEY_CALL_ARGS: &str = "args";
/// Canonical SANDBOX module key. Mirrors `SubgraphBuilder::sandbox`'s
/// `"module"` property.
const KEY_SANDBOX_MODULE: &str = "module";
/// Canonical EMIT topic key.
const KEY_EMIT_TOPIC: &str = "topic";
/// Canonical SUBSCRIBE pattern key.
const KEY_SUBSCRIBE_PATTERN: &str = "pattern";
/// Canonical ITERATE body key. The DSL captures an opaque body *expression*
/// (Text), which is a DISTINCT concept from `SubgraphBuilder::iterate`'s
/// numeric `max` bound (Int) — they are not the same payload, so this is NOT
/// emitted as `"max"`. A distinct non-underscore key keeps #782's overload
/// closed (different from TRANSFORM's `"body"`) without a false semantic
/// equation. (Within-scheme-(a) naming refinement; see design-wireformat-1.)
const KEY_ITERATE_BODY: &str = "iter_body";

// ---------------------------------------------------------------------------
// Public surface
// ---------------------------------------------------------------------------

/// A compiled DSL handler. Carries both the canonical [`Subgraph`] (the
/// shape the engine consumes via `register_subgraph`) and the per-primitive
/// `properties` bags collected from the DSL source.
///
/// The properties bags ARE folded into each `Subgraph` node's `properties`
/// field, so the canonical-bytes encoding (and therefore the CID) reflects
/// the per-primitive config. Devserver consumers may also inspect the
/// `primitives` list directly — same data, different surface.
#[derive(Debug, Clone)]
pub struct CompiledSubgraph {
    /// Canonical Subgraph the engine consumes.
    pub subgraph: Subgraph,
    /// Per-primitive declaration list (id, kind, properties bag) for
    /// devserver introspection. Mirrors the `subgraph.nodes()` order.
    pub primitives: Vec<CompiledPrimitive>,
}

/// One primitive declaration emitted by the DSL parser.
#[derive(Debug, Clone)]
pub struct CompiledPrimitive {
    /// Which of the 12 operation primitives this entry represents.
    pub kind: PrimitiveKind,
    /// Per-primitive configuration bag. Sorted by key (BTreeMap iteration
    /// is ordered) so canonical-bytes encode is permutation-stable.
    pub properties: std::collections::BTreeMap<String, Value>,
}

/// Maximum DSL source length accepted by [`compile_str`] and [`compile_file`]
/// (defense-in-depth — see #545; sibling to the bounded-recursion guard at
/// `parse_object`/`parse_value` for #496). 1 MiB is ~500× the largest in-tree
/// DSL fixture (~2 KiB); the devserver/REPL authoring surface is the only
/// in-tree caller, so this is a guard against accidentally piping a
/// large/binary file into `compile_file`, not an adversarial-network
/// threat. Rejects with [`CompileError::Parse`] carrying
/// [`E_DSL_PARSE_ERROR`] (reuses existing code — no new cross-language-
/// mirrored ErrorCode per §3.5g).
pub const MAX_SOURCE_LEN: usize = 1_048_576;

/// Compile a DSL source string into a [`CompiledSubgraph`].
///
/// # Errors
///
/// Returns [`CompileError`] for any parse, semantic, or build-phase failure.
/// Each diagnostic-carrying error carries a [`Diagnostic`] with [`Span`] +
/// human-readable message + typed `error_code` for devserver rendering.
/// Sources larger
/// than [`MAX_SOURCE_LEN`] are rejected at entry with
/// [`CompileError::Parse`] (#545 defense-in-depth cap).
pub fn compile_str(source: &str) -> Result<CompiledSubgraph, CompileError> {
    if source.len() > MAX_SOURCE_LEN {
        return Err(CompileError::Parse(Diagnostic {
            error_code: E_DSL_PARSE_ERROR,
            message: format!(
                "DSL source length {} bytes exceeds MAX_SOURCE_LEN ({} bytes); \
                 see #545 — splitting/trimming the source is the intended fix",
                source.len(),
                MAX_SOURCE_LEN
            ),
            span: None,
        }));
    }
    if source.trim().is_empty() {
        return Err(CompileError::Parse(Diagnostic {
            error_code: E_DSL_PARSE_ERROR,
            message: "empty DSL source".to_string(),
            span: None,
        }));
    }
    let mut parser = Parser::new(source);
    let handler = parser.parse_handler()?;
    build(handler)
}

/// Compile a DSL source file into a [`CompiledSubgraph`].
///
/// # Errors
///
/// Returns [`CompileError::Io`] for IO failures, including a typed
/// rejection when the on-disk file size exceeds [`MAX_SOURCE_LEN`] (#545
/// defense-in-depth — short-circuits BEFORE [`std::fs::read_to_string`]
/// allocates the buffer). Also propagates any failure modes of
/// [`compile_str`].
pub fn compile_file(path: &Path) -> Result<CompiledSubgraph, CompileError> {
    // #545 short-circuit: check on-disk size BEFORE read_to_string slurps the
    // file into memory; protects against accidentally pointing the devserver
    // at a multi-GB / symlinked binary file.
    let metadata = std::fs::metadata(path)
        .map_err(|e| CompileError::Io(format!("{}: {}", path.display(), e)))?;
    if metadata.len() > MAX_SOURCE_LEN as u64 {
        return Err(CompileError::Io(format!(
            "{}: file size {} bytes exceeds MAX_SOURCE_LEN ({} bytes); \
             see #545 — splitting the source or pointing at a smaller file is the intended fix",
            path.display(),
            metadata.len(),
            MAX_SOURCE_LEN
        )));
    }
    let src = std::fs::read_to_string(path)
        .map_err(|e| CompileError::Io(format!("{}: {}", path.display(), e)))?;
    compile_str(&src)
}

/// Typed compile-error enum surfaced to devserver + downstream tools.
///
/// Wire-stable variant set: each variant maps to a stable `error_code`
/// string (see [`Diagnostic::error_code`]) so devserver / TS-side renderers
/// can switch on the discriminant without prose-string parsing.
///
/// **G-CORE-DSL chunk-3 (closes #790 + #839):** the prior 4-variant set
/// (`Parse` / `Semantic` / `Emit` / `Io`) was renamed + extended:
/// - `Emit` → `Build` — the prior name overloaded the `PrimitiveKind::Emit`
///   operation primitive (CLAUDE.md #1's 12-primitive name; not renameable)
///   AND the `fn emit` internal-emission function (also renamed to `build`).
///   The discriminant now names the COMPILATION PHASE (AST→Subgraph
///   construction), removing the overload with the runtime `Emit`
///   primitive whose ErrorCode is wholly orthogonal.
/// - `Backend(String)` — NEW 5th variant — the typed home for
///   downstream-consumer-injected post-compile rejections (canonical
///   example: `Engine::register_subgraph` returned an error after a
///   successful compile in the devserver flow). Pre-#839 this path
///   abused `CompileError::Io` to wrap engine-registration errors,
///   widening `Io`'s documented semantic ("IO failure reading a source
///   file") to "everything else." The new variant gives downstream
///   consumers a typed bucket distinct from real `std::io::Error`
///   failures and surfaces `E_DSL_BACKEND_REJECTED` on the wire so
///   discriminant-switching consumers route engine-registration
///   failures correctly (not into IO-handling code paths).
#[derive(Debug, Clone, Error)]
pub enum CompileError {
    /// Lexer / parser failure — DSL did not match the grammar.
    #[error("DSL parse error: {0}")]
    Parse(Diagnostic),
    /// Semantic / type-check failure — DSL parsed but referenced unknown
    /// primitives / props / handler ids.
    #[error("DSL semantic error: {0}")]
    Semantic(Diagnostic),
    /// **Compilation-build phase failure** — well-typed AST but
    /// `Subgraph` construction rejected (e.g. missing RESPOND, malformed
    /// structural shape from `validate_shapes`). #790 closure: renamed
    /// from the prior `Emit` variant to disambiguate from
    /// `PrimitiveKind::Emit` (the runtime operation primitive that
    /// publishes events to the engine event bus — a wholly distinct
    /// concept). The error_code strings (`E_DSL_MISSING_RESPOND` /
    /// `E_DSL_INVALID_SHAPE`) are unchanged; the wire-stable load-bearing
    /// stability axis is the error_code, not the variant name.
    #[error("DSL build error: {0}")]
    Build(Diagnostic),
    /// IO failure reading a source file (only from [`compile_file`]).
    /// Pre-#839 this variant was abused by downstream consumers (notably
    /// the devserver) to wrap non-IO engine-registration failures. After
    /// #839 the variant's documented semantic is preserved: this is
    /// reserved for real `std::io::Error` failures from
    /// [`compile_file`]. Downstream consumers wrapping post-compile
    /// rejections should use [`CompileError::Backend`] instead.
    #[error("DSL io error: {0}")]
    Io(String),
    /// **Downstream-consumer-injected post-compile rejection** (#839
    /// closure). The DSL compiler itself never emits this variant —
    /// the compile pipeline emits `Parse` / `Semantic` / `Build` / `Io`
    /// only. Downstream consumers (canonical example: the devserver
    /// engine-registration step at
    /// `tools/benten-dev::DevServer::replace_handler_from_dsl_with_outcome`)
    /// wrap their typed rejections here so discriminant-switching
    /// consumers can route them distinct from IO failures. Maps to
    /// [`E_DSL_BACKEND_REJECTED`] on the wire (typed mirror at
    /// `benten_errors::ErrorCode::DslBackendRejected`).
    #[error("DSL backend rejection: {0}")]
    Backend(String),
}

impl CompileError {
    /// Borrow the inner [`Diagnostic`] when present.
    ///
    /// `Io` and `Backend` variants carry free-form strings (not
    /// [`Diagnostic`]s) because they are downstream-environment failures
    /// rather than source-position-anchored DSL diagnostics; they return
    /// `None` here. Discriminant-switching consumers needing typed wire
    /// codes for those variants should match on the variant directly +
    /// surface the stable codes ([`E_DSL_IO_ERROR`] /
    /// [`E_DSL_BACKEND_REJECTED`]).
    #[must_use]
    pub fn diagnostic(&self) -> Option<&Diagnostic> {
        match self {
            Self::Parse(d) | Self::Semantic(d) | Self::Build(d) => Some(d),
            Self::Io(_) | Self::Backend(_) => None,
        }
    }

    /// Return the stable error-code string for the variant.
    ///
    /// For variants carrying a [`Diagnostic`], returns the inner
    /// [`Diagnostic::error_code`]. For [`CompileError::Io`] returns
    /// [`E_DSL_IO_ERROR`]; for [`CompileError::Backend`] returns
    /// [`E_DSL_BACKEND_REJECTED`]. This is the surface
    /// discriminant-switching consumers (devserver / napi binding / TS
    /// mirror) read to dispatch typed handling without prose-string
    /// parsing.
    #[must_use]
    pub fn error_code(&self) -> &'static str {
        match self {
            Self::Parse(d) | Self::Semantic(d) | Self::Build(d) => d.error_code,
            Self::Io(_) => E_DSL_IO_ERROR,
            Self::Backend(_) => E_DSL_BACKEND_REJECTED,
        }
    }
}

/// Source-span shape — a half-open `[start_offset, end_offset)` byte range
/// into the DSL source, paired with 1-indexed `(line, column)` coordinates
/// at both endpoints for human-readable rendering.
///
/// **G-CORE-DSL chunk-3 (closes #1000):** the prior [`Diagnostic`]
/// carried only a single 1-indexed `(line, column)` cursor point — a
/// point-span, adequate for human-typing-rate devserver rendering but
/// impoverished for AI-loop-feedback (Phase 6 AI-agent ecosystem — the
/// repair-prompt loop slices offending bytes via `start_offset` /
/// `end_offset`, multi-line span rendering needs `(end_line,
/// end_column)`). The new `Span` carries all four coordinates so:
///
/// - editor surfaces (LSP-shaped consumers, devserver squiggle UI) can
///   highlight the offending range, not just place a cursor;
/// - AI repair-prompt loops (per CLAUDE.md baked-in #18 plugins-as-
///   subgraphs + Phase 6 AI-agent trajectory) can slice
///   `source[start_offset..end_offset]` to feed the offender directly
///   back to the model without re-walking the parser;
/// - multi-line constructs (unbalanced parens, long branch bodies)
///   render correctly across the full span instead of pointing only at
///   the open OR close.
///
/// All four coordinates describe the SAME range. `start_offset` /
/// `end_offset` are byte offsets into the source (NOT char offsets;
/// the parser tracks bytes per UTF-8 advance). Half-open: `end_offset`
/// is exclusive, so `source[start_offset..end_offset]` is the offending
/// slice. For point-spans (the "expected X here" cursor case),
/// `start_offset == end_offset` AND `(start_line, start_column) ==
/// (end_line, end_column)`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    /// 1-indexed line of the span's start.
    pub start_line: u32,
    /// 1-indexed column of the span's start.
    pub start_column: u32,
    /// 1-indexed line of the span's end (inclusive line; the cursor
    /// SITS on this line at `end_column`).
    pub end_line: u32,
    /// 1-indexed column of the span's end (the post-token cursor
    /// column, half-open: NOT a character position the offending range
    /// includes).
    pub end_column: u32,
    /// Byte offset into the source where the span starts (inclusive).
    pub start_offset: u32,
    /// Byte offset into the source where the span ends (exclusive,
    /// half-open). `source[start_offset..end_offset]` is the offending
    /// slice.
    pub end_offset: u32,
}

impl Span {
    /// Construct a point-span at a single 1-indexed `(line, column)` +
    /// byte offset (`start == end`).
    #[must_use]
    pub fn point(line: u32, column: u32, offset: u32) -> Self {
        Self {
            start_line: line,
            start_column: column,
            end_line: line,
            end_column: column,
            start_offset: offset,
            end_offset: offset,
        }
    }
}

/// Diagnostic shape devserver renders: span + message + typed error code.
///
/// **G-CORE-DSL chunk-3 (closes #1000):** the prior `(Option<u32>,
/// Option<u32>)` point-span is replaced by an `Option<Span>` carrying
/// the full half-open byte range + endpoint `(line, column)` coordinates.
/// Some diagnostics have no source-position anchor (empty source,
/// MAX_SOURCE_LEN overrun, missing-respond) and continue to carry
/// `span: None`.
#[derive(Debug, Clone)]
pub struct Diagnostic {
    /// Stable error-code string (e.g. `"E_DSL_PARSE_ERROR"`); switch-keyed
    /// by devserver renderer.
    pub error_code: &'static str,
    /// Human-readable message for tooltip + log surface.
    pub message: String,
    /// The source-span the diagnostic anchors at (None if the
    /// diagnostic has no source-position anchor, e.g. empty source).
    pub span: Option<Span>,
}

impl Diagnostic {
    /// Backward-readable accessor: the 1-indexed start-line of the
    /// diagnostic's span, if any. Convenience for callers (devserver
    /// renderer / napi binding) that only need the start coordinate.
    #[must_use]
    pub fn line(&self) -> Option<u32> {
        self.span.map(|s| s.start_line)
    }

    /// Backward-readable accessor: the 1-indexed start-column of the
    /// diagnostic's span, if any.
    #[must_use]
    pub fn column(&self) -> Option<u32> {
        self.span.map(|s| s.start_column)
    }
}

impl std::fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.span {
            Some(s) if s.start_line == s.end_line && s.start_column == s.end_column => {
                // Point-span: render `line:col` for compactness.
                write!(
                    f,
                    "[{}] {}:{} {}",
                    self.error_code, s.start_line, s.start_column, self.message
                )
            }
            Some(s) => {
                // Range-span: render `startLine:startCol-endLine:endCol`
                // so multi-line spans (unbalanced parens, long bodies)
                // are visible at a glance.
                write!(
                    f,
                    "[{}] {}:{}-{}:{} {}",
                    self.error_code,
                    s.start_line,
                    s.start_column,
                    s.end_line,
                    s.end_column,
                    self.message
                )
            }
            None => write!(f, "[{}] {}", self.error_code, self.message),
        }
    }
}

// ---------------------------------------------------------------------------
// Stable error codes
// ---------------------------------------------------------------------------

/// Typed error-code constants surfaced via [`Diagnostic::error_code`].
/// Promoted from `pub(crate)` → `pub` per #841 so downstream consumers
/// (devserver / `bindings/napi/src/devserver.rs` / TS mirror) can match
/// against the typed constants instead of duplicated string literals.
/// `&'static str` is preserved because [`Diagnostic::error_code`] is
/// declared `&'static str` (a wire-stability constraint from the
/// pre-v1-API-stabilization window; an enum-typed code is a parallel
/// possible refactor recorded at #841 but kept out of scope here to
/// keep the v1-API surface narrow). Cross-language mirror per §3.5g:
/// the TS side reads these as the typed `EDsl*` BentenError subclasses
/// at `packages/engine/src/dsl.ts`.
pub const E_DSL_PARSE_ERROR: &str = "E_DSL_PARSE_ERROR";
/// See [`E_DSL_PARSE_ERROR`] for the typed-constant promotion rationale.
pub const E_DSL_UNKNOWN_PRIMITIVE: &str = "E_DSL_UNKNOWN_PRIMITIVE";
/// See [`E_DSL_PARSE_ERROR`] for the typed-constant promotion rationale.
pub const E_DSL_MISSING_RESPOND: &str = "E_DSL_MISSING_RESPOND";
/// Phase-3 R6 fp Wave C2 (closes dx-r6-r1-1 MAJOR — DSL orphan code half):
/// shape validation rejected a primitive's typed property (e.g. SANDBOX
/// `fuel` declared as a string instead of an integer). Mirrors the
/// TS-side `EDslInvalidShape` thrown from `packages/engine/src/dsl.ts`
/// builder methods so a Rust callsite emitting this surfaces the same
/// typed `BentenError` subclass on the wire. Drift-detect reachability
/// path: `crates/benten-dsl-compiler/src/lib.rs::validate_shapes` (a
/// crate-private free function, NOT a member of a `build` module —
/// `build` and `validate_shapes` are sibling free functions; the prior
/// `emit` framing was renamed at G-CORE-DSL chunk-3 #790).
pub const E_DSL_INVALID_SHAPE: &str = "E_DSL_INVALID_SHAPE";
/// G-CORE-DSL chunk-3 (closes #839 abuse): stable error-code string
/// reported by [`CompileError::error_code`] for the `Io` variant.
/// Distinct from [`E_DSL_BACKEND_REJECTED`] (which is the
/// downstream-consumer post-compile rejection bucket). The `Io` variant
/// itself carries a free-form `String` (the original
/// `std::io::Error::to_string()`), not a [`Diagnostic`], so this code
/// is surfaced via `CompileError::error_code` rather than via a
/// `Diagnostic::error_code` field.
pub const E_DSL_IO_ERROR: &str = "E_DSL_IO_ERROR";
/// G-CORE-DSL chunk-3 (closes #839): stable error-code string for the
/// new [`CompileError::Backend`] variant. Downstream consumers wrap
/// their typed post-compile rejections (canonical example:
/// `Engine::register_subgraph` failure in the devserver flow) with
/// this code so discriminant-switching consumers route them distinct
/// from real `Io` failures. Cross-language mirror (§3.5g) lives at
/// `benten_errors::ErrorCode::DslBackendRejected`.
pub const E_DSL_BACKEND_REJECTED: &str = "E_DSL_BACKEND_REJECTED";

// ---------------------------------------------------------------------------
// AST
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub(crate) struct HandlerAst {
    pub handler_id: String,
    pub primitives: Vec<PrimitiveAst>,
}

#[derive(Debug, Clone)]
pub(crate) struct PrimitiveAst {
    pub kind: PrimitiveKind,
    /// Per-primitive properties collected at parse-time.
    pub properties: std::collections::BTreeMap<String, Value>,
}

// ---------------------------------------------------------------------------
// Parser — hand-written, single-pass, line/column-tracking.
// ---------------------------------------------------------------------------

struct Parser<'a> {
    src: &'a str,
    /// Byte offset into `src`.
    pos: usize,
    /// 1-indexed current line.
    line: u32,
    /// 1-indexed current column.
    column: u32,
}

impl<'a> Parser<'a> {
    fn new(src: &'a str) -> Self {
        Self {
            src,
            pos: 0,
            line: 1,
            column: 1,
        }
    }

    fn parse_handler(&mut self) -> Result<HandlerAst, CompileError> {
        self.skip_ws();
        self.expect_keyword("handler")?;
        self.skip_ws();
        let handler_id = self.parse_string()?;
        self.skip_ws();
        self.expect_char('{')?;
        let mut primitives = Vec::new();
        loop {
            self.skip_ws();
            primitives.push(self.parse_primitive()?);
            self.skip_ws();
            // #931 closure: byte-comparison for the `->` chain operator's
            // 2-byte ASCII lookahead instead of the prior `peek_at(1)` call
            // which walked the UTF-8 char-decoder twice. Both `-` and `>` are
            // ASCII so byte-comparison is equivalent to char-comparison.
            // `peek_at` is removed entirely (sole callsite).
            let bytes = self.src.as_bytes();
            if bytes.get(self.pos) == Some(&b'-') && bytes.get(self.pos + 1) == Some(&b'>') {
                self.advance(); // -
                self.advance(); // >
                continue;
            }
            break;
        }
        self.skip_ws();
        self.expect_char('}')?;
        Ok(HandlerAst {
            handler_id,
            primitives,
        })
    }

    /// #663 closure: parse `'(' STRING ')'` shape used by `read` / `emit` /
    /// `subscribe` / `stream`. Returns the inner STRING.
    fn parse_paren_string(&mut self) -> Result<String, CompileError> {
        self.skip_ws();
        self.expect_char('(')?;
        self.skip_ws();
        let s = self.parse_string()?;
        self.skip_ws();
        self.expect_char(')')?;
        Ok(s)
    }

    /// #663 closure: parse `'(' STRING ( ',' object )? ')'` shape used by
    /// `write` / `call` / `sandbox`. Returns the inner STRING + the optional
    /// post-comma object.
    fn parse_paren_string_optional_object(
        &mut self,
    ) -> Result<(String, Option<std::collections::BTreeMap<String, Value>>), CompileError> {
        self.skip_ws();
        self.expect_char('(')?;
        self.skip_ws();
        let s = self.parse_string()?;
        self.skip_ws();
        let obj = if self.peek() == Some(',') {
            self.advance();
            self.skip_ws();
            let o = self.parse_object()?;
            self.skip_ws();
            Some(o)
        } else {
            None
        };
        self.expect_char(')')?;
        Ok((s, obj))
    }

    /// #663 closure: parse `'(' object ')'` shape used by `transform` /
    /// `wait`. Returns the inner object.
    fn parse_paren_object(
        &mut self,
    ) -> Result<std::collections::BTreeMap<String, Value>, CompileError> {
        self.skip_ws();
        self.expect_char('(')?;
        self.skip_ws();
        let o = self.parse_object()?;
        self.skip_ws();
        self.expect_char(')')?;
        Ok(o)
    }

    #[allow(
        clippy::too_many_lines,
        reason = "12-primitive keyword-dispatch match must stay as a single table for readability; splitting per-arm scatters the call shape without making the table easier to read (rationale carried forward from the prior `parse_primitive` allow at chunk-2)"
    )]
    fn parse_primitive(&mut self) -> Result<PrimitiveAst, CompileError> {
        let (start_line, start_col, start_off) = (self.line, self.column, self.pos);
        let ident = self.parse_identifier()?;
        let mut props = std::collections::BTreeMap::<String, Value>::new();
        // #663 closure: the prior 154-LOC inline dispatch table is collapsed
        // to ~50 LOC via three shared parse helpers (`parse_paren_string` /
        // `parse_paren_string_optional_object` / `parse_paren_object`) that
        // capture the 3 recurring grammar shapes across the 12 primitives.
        // The `#[allow(clippy::too_many_lines)]` attribute is no longer
        // needed at this scale. The 3 outliers (`branch` + `iterate` use
        // `read_until_balanced`; `respond` is the single-token form) remain
        // inline. Per-primitive property keys + outputs are byte-identical
        // to the prior dispatch (verified by the existing test suite +
        // `permuted_keys_yield_identical_canonical_bytes`).
        let kind = match ident.as_str() {
            "read" => {
                let label = self.parse_paren_string()?;
                props.insert(KEY_LABEL.to_string(), Value::Text(label));
                PrimitiveKind::Read
            }
            "write" => {
                let (label, body) = self.parse_paren_string_optional_object()?;
                props.insert(KEY_LABEL.to_string(), Value::Text(label));
                if let Some(b) = body {
                    props.insert(KEY_USER_PROPERTIES.to_string(), Value::Map(b));
                }
                PrimitiveKind::Write
            }
            "transform" => {
                let body = self.parse_paren_object()?;
                props.insert(KEY_TRANSFORM_BODY.to_string(), Value::Map(body));
                PrimitiveKind::Transform
            }
            "branch" => {
                self.skip_ws();
                self.expect_char('(')?;
                let expr = self.read_until_balanced(')')?;
                props.insert(
                    KEY_PREDICATE.to_string(),
                    Value::Text(expr.trim().to_string()),
                );
                self.expect_char(')')?;
                PrimitiveKind::Branch
            }
            "wait" => {
                let body = self.parse_paren_object()?;
                for (k, v) in body {
                    props.insert(k, v);
                }
                PrimitiveKind::Wait
            }
            "call" => {
                let (target, body) = self.parse_paren_string_optional_object()?;
                props.insert(KEY_CALL_HANDLER.to_string(), Value::Text(target));
                if let Some(b) = body {
                    props.insert(KEY_CALL_ARGS.to_string(), Value::Map(b));
                }
                PrimitiveKind::Call
            }
            "sandbox" => {
                let (module, body) = self.parse_paren_string_optional_object()?;
                props.insert(KEY_SANDBOX_MODULE.to_string(), Value::Text(module));
                if let Some(b) = body {
                    for (k, v) in b {
                        props.insert(k, v);
                    }
                }
                PrimitiveKind::Sandbox
            }
            "respond" => PrimitiveKind::Respond,
            "emit" => {
                let topic = self.parse_paren_string()?;
                props.insert(KEY_EMIT_TOPIC.to_string(), Value::Text(topic));
                PrimitiveKind::Emit
            }
            "subscribe" => {
                let pattern = self.parse_paren_string()?;
                props.insert(KEY_SUBSCRIBE_PATTERN.to_string(), Value::Text(pattern));
                PrimitiveKind::Subscribe
            }
            "stream" => {
                let label = self.parse_paren_string()?;
                props.insert(KEY_LABEL.to_string(), Value::Text(label));
                PrimitiveKind::Stream
            }
            "iterate" => {
                self.skip_ws();
                self.expect_char('(')?;
                let body = self.read_until_balanced(')')?;
                props.insert(
                    KEY_ITERATE_BODY.to_string(),
                    Value::Text(body.trim().to_string()),
                );
                self.expect_char(')')?;
                PrimitiveKind::Iterate
            }
            other => {
                // #1000 closure: span covers the offending keyword
                // from start (captured pre-parse_identifier) to current
                // cursor (post-parse_identifier consumed the ident).
                return Err(CompileError::Semantic(Diagnostic {
                    error_code: E_DSL_UNKNOWN_PRIMITIVE,
                    message: format!("unknown primitive `{other}`"),
                    span: Some(Span {
                        start_line,
                        start_column: start_col,
                        end_line: self.line,
                        end_column: self.column,
                        start_offset: start_off as u32,
                        end_offset: self.pos as u32,
                    }),
                }));
            }
        };
        Ok(PrimitiveAst {
            kind,
            properties: props,
        })
    }

    fn parse_object(&mut self) -> Result<std::collections::BTreeMap<String, Value>, CompileError> {
        self.expect_char('{')?;
        let mut map = std::collections::BTreeMap::new();
        loop {
            self.skip_ws();
            if self.peek() == Some('}') {
                self.advance();
                break;
            }
            let key = self.parse_identifier()?;
            self.skip_ws();
            self.expect_char(':')?;
            self.skip_ws();
            let value = self.parse_value()?;
            map.insert(key, value);
            self.skip_ws();
            // #760 closure: capture span-start BEFORE peek so the
            // "expected ',' or '}'" diagnostic points at the offending
            // unexpected character rather than at the post-skip_ws cursor.
            // #1000 closure: also capture byte offset for the new Span
            // shape; end_offset is the post-peek cursor (= start_off for
            // point-spans when no token is consumed past start).
            let (sl, sc, so) = (self.line, self.column, self.pos);
            match self.peek() {
                Some(',') => {
                    self.advance();
                }
                Some('}') => {
                    self.advance();
                    break;
                }
                _ => return Err(self.parse_err_at(sl, sc, so, "expected ',' or '}'".to_string())),
            }
        }
        Ok(map)
    }

    fn parse_value(&mut self) -> Result<Value, CompileError> {
        self.skip_ws();
        // #760 closure: capture span-start at value-start (post skip_ws,
        // pre any consume) so all parse_value diagnostics point at the
        // start of the offending value rather than at the post-token cursor
        // (which for numeric / identifier / "expected value" cases was
        // misleading by the value's full byte-width).
        // #1000 closure: also capture byte offset for the new Span shape.
        let (start_line, start_col, start_off) = (self.line, self.column, self.pos);
        match self.peek() {
            Some('\'') => Ok(Value::Text(self.parse_string()?)),
            Some('{') => Ok(Value::Map(self.parse_object()?)),
            Some('$') => {
                // Variable reference — preserve as `$path.dotted`.
                // #934 closure: capture the leading `$` in the source-slice
                // window instead of bootstrapping a fresh `String::from("$")`
                // + char-by-char `push`. The cursor sits ON the `$` here, so
                // `start` captures it; the inner loop advances over the
                // identifier-path tail, and the final `&self.src[start..]`
                // slice already includes the leading `$`. Removes 1
                // allocation + N realloc-grow steps per variable reference.
                let start = self.pos;
                self.advance(); // consume the `$`
                while let Some(c) = self.peek() {
                    if c.is_ascii_alphanumeric() || c == '_' || c == '.' {
                        self.advance();
                    } else {
                        break;
                    }
                }
                Ok(Value::Text(self.src[start..self.pos].to_string()))
            }
            Some(c) if c.is_ascii_digit() || c == '-' => {
                // #934 closure: parse numeric literals by slicing the source
                // window directly instead of building a fresh `String` +
                // pushing char-by-char. `i64::from_str` and `f64::from_str`
                // both accept `&str`. Removes 1 `String` allocation + N
                // realloc-grow steps per numeric literal. Same pattern as
                // `Parser::parse_identifier` + `Parser::read_until_balanced`.
                // `parse_string` is intentionally NOT migrated because it
                // would have to handle quote-escapes.
                let start = self.pos;
                if c == '-' {
                    self.advance();
                }
                let mut saw_dot = false;
                while let Some(c) = self.peek() {
                    if c.is_ascii_digit() {
                        self.advance();
                    } else if c == '.' && !saw_dot {
                        saw_dot = true;
                        self.advance();
                    } else {
                        break;
                    }
                }
                let slice = &self.src[start..self.pos];
                if saw_dot {
                    let v: f64 = slice.parse().map_err(|_| {
                        self.parse_err_at(
                            start_line,
                            start_col,
                            start_off,
                            format!("invalid float `{slice}`"),
                        )
                    })?;
                    Ok(Value::Float(v))
                } else {
                    let v: i64 = slice.parse().map_err(|_| {
                        self.parse_err_at(
                            start_line,
                            start_col,
                            start_off,
                            format!("invalid int `{slice}`"),
                        )
                    })?;
                    Ok(Value::Int(v))
                }
            }
            Some('t' | 'f') => {
                let ident = self.parse_identifier()?;
                match ident.as_str() {
                    "true" => Ok(Value::Bool(true)),
                    "false" => Ok(Value::Bool(false)),
                    other => Err(self.parse_err_at(
                        start_line,
                        start_col,
                        start_off,
                        format!("unexpected identifier `{other}`"),
                    )),
                }
            }
            _ => Err(self.parse_err_at(
                start_line,
                start_col,
                start_off,
                "expected value".to_string(),
            )),
        }
    }

    // -- token helpers --

    fn parse_string(&mut self) -> Result<String, CompileError> {
        // #760 closure: capture span-start at the opening quote so the
        // "unterminated string" diagnostic points at the string's start
        // (where the user opened the quote) rather than at the end-of-input
        // cursor. This is the most-useful target for the only diagnostic
        // this function can emit.
        // #1000 closure: also capture byte offset for the Span shape.
        let (start_line, start_col, start_off) = (self.line, self.column, self.pos);
        self.expect_char('\'')?;
        let mut s = String::new();
        loop {
            match self.peek() {
                Some('\'') => {
                    self.advance();
                    return Ok(s);
                }
                Some(c) => {
                    s.push(c);
                    self.advance();
                }
                None => {
                    return Err(self.parse_err_at(
                        start_line,
                        start_col,
                        start_off,
                        "unterminated string".to_string(),
                    ));
                }
            }
        }
    }

    fn parse_identifier(&mut self) -> Result<String, CompileError> {
        let start = self.pos;
        let (start_line, start_col) = (self.line, self.column);
        while let Some(c) = self.peek() {
            if c.is_ascii_alphanumeric() || c == '_' {
                self.advance();
            } else {
                break;
            }
        }
        if self.pos == start {
            // #1000 closure: point-span at the cursor (no token
            // consumed). end == start.
            return Err(CompileError::Parse(Diagnostic {
                error_code: E_DSL_PARSE_ERROR,
                message: "expected identifier".to_string(),
                span: Some(Span::point(start_line, start_col, start as u32)),
            }));
        }
        Ok(self.src[start..self.pos].to_string())
    }

    fn expect_keyword(&mut self, kw: &str) -> Result<(), CompileError> {
        // #760 closure: capture span-start at the keyword's first character
        // so the "expected keyword X, got Y" diagnostic points at the
        // offending token's start, not at the post-identifier cursor.
        // #1000 closure: also capture byte offset; end-of-span is the
        // post-parse_identifier cursor (covers the offending ident).
        let (start_line, start_col, start_off) = (self.line, self.column, self.pos);
        let id = self.parse_identifier()?;
        if id == kw {
            Ok(())
        } else {
            // #1000: end-of-span is the current cursor (post-ident).
            Err(CompileError::Parse(Diagnostic {
                error_code: E_DSL_PARSE_ERROR,
                message: format!("expected keyword `{kw}`, got `{id}`"),
                span: Some(Span {
                    start_line,
                    start_column: start_col,
                    end_line: self.line,
                    end_column: self.column,
                    start_offset: start_off as u32,
                    end_offset: self.pos as u32,
                }),
            }))
        }
    }

    fn expect_char(&mut self, c: char) -> Result<(), CompileError> {
        // #760 closure: capture span-start at the offending character
        // BEFORE peek so both "got `X`" and "got end-of-input" diagnostics
        // point at the position where the expected char SHOULD have been
        // (which IS self.line/self.column at fn-entry — but capturing
        // explicitly makes the intent reader-obvious + matches the rest of
        // the #760-touched call sites).
        // #1000 closure: also capture byte offset.
        let (start_line, start_col, start_off) = (self.line, self.column, self.pos);
        match self.peek() {
            Some(p) if p == c => {
                self.advance();
                Ok(())
            }
            Some(p) => Err(self.parse_err_at(
                start_line,
                start_col,
                start_off,
                format!("expected `{c}`, got `{p}`"),
            )),
            None => Err(self.parse_err_at(
                start_line,
                start_col,
                start_off,
                format!("expected `{c}`, got end-of-input"),
            )),
        }
    }

    fn read_until_balanced(&mut self, close: char) -> Result<String, CompileError> {
        // Treat `(` / `)` parens balancing for a `branch(...)` or
        // `iterate(...)` expression body. Handles nested parens; stops at
        // the unbalanced `close` char without consuming it.
        let start = self.pos;
        // #760 closure: capture span-start at the expression body's start
        // so the "expected `close`, hit end-of-input" diagnostic points at
        // where the unbalanced expression OPENED rather than at end-of-input
        // (much more useful when a `branch(...)` paren never closes deep in
        // a chain).
        let (start_line, start_col) = (self.line, self.column);
        let mut depth: i32 = 0;
        loop {
            match self.peek() {
                Some(c) if c == close && depth == 0 => break,
                Some('(') => {
                    depth += 1;
                    self.advance();
                }
                Some(')') => {
                    depth -= 1;
                    self.advance();
                }
                Some(_) => self.advance(),
                None => {
                    // #1000: end-of-span covers the WHOLE walked body
                    // (start..self.pos) so the AI repair-prompt loop can
                    // slice the offending expression directly. This is
                    // load-bearing for the unbalanced-paren case: the
                    // span covers everything from the open paren to the
                    // EOF where the close paren never arrived.
                    return Err(CompileError::Parse(Diagnostic {
                        error_code: E_DSL_PARSE_ERROR,
                        message: format!("expected `{close}`, hit end-of-input"),
                        span: Some(Span {
                            start_line,
                            start_column: start_col,
                            end_line: self.line,
                            end_column: self.column,
                            start_offset: start as u32,
                            end_offset: self.pos as u32,
                        }),
                    }));
                }
            }
        }
        Ok(self.src[start..self.pos].to_string())
    }

    /// #760 closure: span-anchored parse-error constructor. All parse-error
    /// callsites use this — callers capture `(self.line, self.column,
    /// self.pos)` BEFORE the consume-step and pass the captured triple
    /// here so the diagnostic's [`Span`] marks the offending span's
    /// *start* rather than the post-token cursor. Most callsites pass
    /// the cursor's current `(line, column, pos)` as start (a point-span
    /// when no token has been consumed past start); `Parser::parse_value`
    /// + `parse_string` + `read_until_balanced` are the wider-span
    /// callers using a captured start from earlier in the parse.
    /// #1000 closure: signature widened to carry `start_offset`; end of
    /// the span is the current cursor at error-construction time so the
    /// span covers any tokens consumed between start-capture and
    /// error-emit (e.g. an invalid numeric literal that walked the full
    /// digit sequence before `i64::from_str` failed).
    fn parse_err_at(
        &self,
        line: u32,
        column: u32,
        start_offset: usize,
        message: String,
    ) -> CompileError {
        CompileError::Parse(Diagnostic {
            error_code: E_DSL_PARSE_ERROR,
            message,
            span: Some(Span {
                start_line: line,
                start_column: column,
                end_line: self.line,
                end_column: self.column,
                start_offset: start_offset as u32,
                end_offset: self.pos as u32,
            }),
        })
    }

    // -- low-level cursor --

    fn peek(&self) -> Option<char> {
        self.src[self.pos..].chars().next()
    }

    fn advance(&mut self) {
        if let Some(c) = self.peek() {
            self.pos += c.len_utf8();
            if c == '\n' {
                self.line += 1;
                self.column = 1;
            } else {
                self.column += 1;
            }
        }
    }

    fn skip_ws(&mut self) {
        loop {
            match self.peek() {
                Some(c) if c.is_whitespace() => self.advance(),
                _ => break,
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Build — AST → benten_core::Subgraph + CompiledPrimitive list.
// ---------------------------------------------------------------------------
//
// #790 closure (G-CORE-DSL chunk-3): renamed from `emit` to `build` to
// disambiguate from `PrimitiveKind::Emit` (the runtime operation
// primitive that publishes events to the engine event bus — CLAUDE.md
// #1 12-primitive-irreducibility commits the name to the runtime
// primitive). The compilation-build phase NOW reads "build the Subgraph
// from the AST" which is the actual operation. Sibling
// `validate_shapes` is unchanged; both remain crate-private free
// functions.

fn build(handler: HandlerAst) -> Result<CompiledSubgraph, CompileError> {
    use benten_core::OperationNode;

    // Shape-validation pass — fires `E_DSL_INVALID_SHAPE` for typed-property
    // mis-declarations a downstream consumer (engine register_subgraph /
    // wasmtime config) would otherwise reject with a less-actionable error.
    // R6 fp Wave C2 (dx-r6-r1-1 MAJOR closure half): brings the Rust dsl-
    // compiler in line with the TS-side `EDslInvalidShape` contract.
    validate_shapes(&handler)?;

    if !handler
        .primitives
        .iter()
        .any(|p| matches!(p.kind, PrimitiveKind::Respond))
    {
        return Err(CompileError::Build(Diagnostic {
            error_code: E_DSL_MISSING_RESPOND,
            message: format!(
                "handler `{}` does not contain a `respond` primitive",
                handler.handler_id
            ),
            span: None,
        }));
    }

    let mut sg = Subgraph::new(handler.handler_id);
    let mut primitives = Vec::with_capacity(handler.primitives.len());
    let mut prev_id: Option<String> = None;
    for (idx, p) in handler.primitives.into_iter().enumerate() {
        let id = id_for(p.kind, idx);
        let mut node = OperationNode::new(&id, p.kind);
        for (k, v) in &p.properties {
            node = node.with_property(k.clone(), v.clone());
        }
        sg = sg.push_node_raw(node);
        if let Some(prev) = &prev_id {
            sg = sg.push_edge_raw(prev, &id, "next");
        }
        primitives.push(CompiledPrimitive {
            kind: p.kind,
            properties: p.properties,
        });
        prev_id = Some(id);
    }

    Ok(CompiledSubgraph {
        subgraph: sg,
        primitives,
    })
}

/// Phase-3 R6 fp Wave C2 (dx-r6-r1-1 MAJOR closure half): shape-validation
/// pass over the parsed AST that catches typed-property mis-declarations
/// before the engine sees them. Surfaces `E_DSL_INVALID_SHAPE` (the
/// catalog-only TS-side code, now a first-class Rust ErrorCode) so JS
/// callers consuming the diagnostic see the same typed `BentenError`
/// subclass (`EDslInvalidShape`) regardless of whether the offending
/// handler was authored via the TS DSL builder or via this Rust dsl-
/// compiler. Today the pass enforces SANDBOX integer-typed properties
/// (`fuel`, `wallclock_ms`, `output_limit`) per
/// `docs/SANDBOX-LIMITS.md`. Property names are the CANONICAL eval-side
/// snake_case form per the 24th-p/c-drift acceptance criterion enforced
/// at `crates/benten-eval/tests/sandbox_handler_args.rs` (the camelCase
/// `wallclockMs` / `outputLimitBytes` TS-surface form is translated by
/// `packages/engine/src/dsl.ts::translateSandboxArgs` to the canonical
/// snake_case form BEFORE crossing the napi boundary; the Rust-side
/// validator therefore validates the canonical names only). Future
/// shape rules append to this single pass so the typed-error surface
/// stays narrow.
fn validate_shapes(handler: &HandlerAst) -> Result<(), CompileError> {
    use benten_core::Value;
    /// SANDBOX numeric-budget property names (per `docs/SANDBOX-LIMITS.md` §2).
    /// Each MUST be a non-negative integer; non-int / negative-int / non-numeric
    /// values trip `E_DSL_INVALID_SHAPE`. Names are the canonical eval-side
    /// snake_case form consumed by
    /// `crates/benten-engine/src/primitive_host.rs::execute_sandbox`.
    const SANDBOX_INT_PROPS: &[&str] = &["fuel", "wallclock_ms", "output_limit"];

    // #608 (safe-3) — handler_id MUST be a non-empty, non-whitespace-only
    // identifier. Cross-language rule-mirror (§3.5g): the TS-side DSL
    // builder (`packages/engine/src/dsl.ts::SubgraphBuilder`) rejects empty handlerIds
    // with `EDslInvalidShape("handlerId must be a non-empty string")`;
    // surface the same typed code on the Rust side so a handler authored
    // via either surface fails identically at compile-time rather than
    // propagating ambiguously to `Engine::register_subgraph`.
    if handler.handler_id.trim().is_empty() {
        return Err(CompileError::Build(Diagnostic {
            error_code: E_DSL_INVALID_SHAPE,
            message: "handler id must be a non-empty, non-whitespace string \
                      (#608 — cross-language mirror with TS-side \
                      EDslInvalidShape)"
                .to_string(),
            span: None,
        }));
    }

    // #671 (qual-1): collapse the two near-identical error-construction arms
    // into a single helper closure capturing the SANDBOX-int-prop typed-error
    // skeleton. Single source of truth for the message body + the typed
    // `E_DSL_INVALID_SHAPE` code; future amendments to the SANDBOX-budget
    // error message touch one site, not two.
    let sandbox_int_err = |key: &str, got: &dyn std::fmt::Display| {
        CompileError::Build(Diagnostic {
            error_code: E_DSL_INVALID_SHAPE,
            message: format!(
                "sandbox primitive `{key}` property must be a non-negative integer (got {got}); see docs/SANDBOX-LIMITS.md §2"
            ),
            span: None,
        })
    };
    for p in &handler.primitives {
        if matches!(p.kind, PrimitiveKind::Sandbox) {
            for &key in SANDBOX_INT_PROPS {
                if let Some(v) = p.properties.get(key) {
                    match v {
                        Value::Int(n) if *n >= 0 => {}
                        Value::Int(n) => return Err(sandbox_int_err(key, n)),
                        other => return Err(sandbox_int_err(key, &format!("{other:?}"))),
                    }
                }
            }
        }
    }
    Ok(())
}

fn id_for(kind: PrimitiveKind, idx: usize) -> String {
    // #798 scheme-(a): uniform 2-char prefixes. The per-node id is hashed
    // into the canonical bytes (`Subgraph::canonical_view` sorts by
    // `(id, kind)`), so the prefix scheme is wire-stable — it MUST be
    // normalized pre-v1. The prior scheme mixed 1-char (`r`/`w`) and
    // 4-char (`wait`/`resp`) prefixes; uniform 2-char removes the
    // irregularity and disambiguates Sandbox/Subscribe/Stream without
    // 3-/4-char outliers.
    let prefix = match kind {
        PrimitiveKind::Read => "re",
        PrimitiveKind::Write => "wr",
        PrimitiveKind::Transform => "tr",
        PrimitiveKind::Branch => "br",
        PrimitiveKind::Iterate => "it",
        PrimitiveKind::Wait => "wt",
        PrimitiveKind::Call => "ca",
        PrimitiveKind::Respond => "rs",
        PrimitiveKind::Emit => "em",
        PrimitiveKind::Sandbox => "sb",
        PrimitiveKind::Subscribe => "su",
        PrimitiveKind::Stream => "sm",
        // #848 (surf-1) — `PrimitiveKind` is `#[non_exhaustive]` upstream
        // for forward-compatibility, but **CLAUDE.md #1 commits the 12
        // operation primitives as irreducible**. A 13th variant landing
        // upstream without re-opening commitment #1 (and without adding a
        // matching DSL keyword + per-variant `id_for` prefix here) would
        // silently collide all unknown variants under the same `op` prefix
        // → CID-instability across new-variant additions (the load-bearing
        // hazard pre-named at `INTERNALS.md` §9 item 5). Make the silent
        // fallback LOUD: panic with a typed reason so the next maintainer
        // sees the structural break at the first call, not as a CID-drift
        // bug-report from production. The DSL parser cannot construct a
        // non-12-variant `PrimitiveKind` at HEAD (the parser dispatch
        // covers exactly the 12 keywords + falls through to
        // `E_DSL_UNKNOWN_PRIMITIVE`), so this branch is unreachable from
        // the public surface today.
        kind => unreachable!(
            "#848: PrimitiveKind variant {kind:?} has no DSL id-prefix mapping. \
             A new variant landed in benten-core without a matching arm here \
             — re-opening CLAUDE.md #1 (12 primitives irreducible) is required \
             before adding the 13th; once the commitment is updated, add a new \
             2-char prefix arm above (mirroring the existing 12 entries).",
        ),
    };
    format!("{prefix}{idx}")
}

#[cfg(test)]
mod inline_tests {
    use super::*;

    #[test]
    fn round_trip_minimal_handler() {
        let src = "handler 'h' { read('post') -> respond }";
        let c = compile_str(src).expect("must compile");
        assert_eq!(c.subgraph.handler_id(), "h");
        assert_eq!(c.primitives.len(), 2);
        assert_eq!(c.primitives[0].kind, PrimitiveKind::Read);
        assert_eq!(c.primitives[1].kind, PrimitiveKind::Respond);
    }

    #[test]
    fn empty_source_is_typed_parse_error() {
        let err = compile_str("").unwrap_err();
        assert!(matches!(err, CompileError::Parse(_)));
        let d = err.diagnostic().unwrap();
        assert_eq!(d.error_code, E_DSL_PARSE_ERROR);
    }

    #[test]
    fn missing_respond_is_typed_build_error() {
        // G-CORE-DSL chunk-3 #790 closure: variant renamed `Emit` → `Build`.
        let err = compile_str("handler 'h' { read('post') }").unwrap_err();
        assert!(matches!(err, CompileError::Build(_)));
        assert_eq!(err.diagnostic().unwrap().error_code, E_DSL_MISSING_RESPOND);
    }

    #[test]
    fn unknown_primitive_is_typed_semantic_error() {
        let err = compile_str("handler 'h' { read('post') -> teleport -> respond }").unwrap_err();
        assert!(matches!(err, CompileError::Semantic(_)));
        assert_eq!(
            err.diagnostic().unwrap().error_code,
            E_DSL_UNKNOWN_PRIMITIVE
        );
    }

    #[test]
    fn unbalanced_brace_is_typed_parse_error() {
        let err = compile_str("handler 'h' { read('post') -> respond").unwrap_err();
        assert!(matches!(err, CompileError::Parse(_)));
    }

    /// R6 fp Wave C2 (closes dx-r6-r1-1 MAJOR — DSL orphan-code half):
    /// SANDBOX `fuel` declared as a string trips the typed
    /// `E_DSL_INVALID_SHAPE` Emit-error rather than surviving to the
    /// engine where wasmtime would surface a less-actionable rejection.
    /// Mirrors the TS-side `EDslInvalidShape` thrown from the dsl.ts
    /// builder methods so JS callers see the same typed BentenError
    /// regardless of which DSL surface authored the handler.
    #[test]
    fn sandbox_fuel_declared_as_string_is_typed_invalid_shape() {
        // G-CORE-DSL chunk-3 #790 closure: variant renamed `Emit` → `Build`.
        let src = "handler 'h' { sandbox('mod', { fuel: 'high' }) -> respond }";
        let err = compile_str(src).unwrap_err();
        assert!(
            matches!(err, CompileError::Build(_)),
            "fuel-as-string trips Build-shape error, got {err:?}"
        );
        assert_eq!(
            err.diagnostic().unwrap().error_code,
            E_DSL_INVALID_SHAPE,
            "must surface E_DSL_INVALID_SHAPE typed code"
        );
    }

    #[test]
    fn sandbox_negative_fuel_is_typed_invalid_shape() {
        // G-CORE-DSL chunk-3 #790 closure: variant renamed `Emit` → `Build`.
        let src = "handler 'h' { sandbox('mod', { fuel: -1 }) -> respond }";
        let err = compile_str(src).unwrap_err();
        assert!(
            matches!(err, CompileError::Build(_)),
            "negative fuel trips Build-shape error, got {err:?}"
        );
        assert_eq!(err.diagnostic().unwrap().error_code, E_DSL_INVALID_SHAPE,);
    }

    #[test]
    fn sandbox_valid_integer_fuel_compiles() {
        // Sanity counterpart — integer fuel is the documented happy path.
        let src = "handler 'h' { sandbox('mod', { fuel: 500000 }) -> respond }";
        let c = compile_str(src).expect("integer fuel must compile");
        assert_eq!(c.primitives.len(), 2);
        assert_eq!(c.primitives[0].kind, PrimitiveKind::Sandbox);
    }

    #[test]
    fn permuted_keys_yield_identical_canonical_bytes() {
        let a = compile_str(
            "handler 'h' { sandbox('m', { wallclock_ms: 30000, output_limit: 65536 }) -> respond }",
        )
        .unwrap();
        let b = compile_str(
            "handler 'h' { sandbox('m', { output_limit: 65536, wallclock_ms: 30000 }) -> respond }",
        )
        .unwrap();
        assert_eq!(
            a.subgraph.to_canonical_bytes().unwrap(),
            b.subgraph.to_canonical_bytes().unwrap(),
            "BTreeMap ordering ensures permutation-stable canonical bytes"
        );
    }

    // -----------------------------------------------------------------
    // G-CORE-DSL chunk-2 closure pins
    // -----------------------------------------------------------------

    /// #934 closure pin: integer-typed property values parse correctly
    /// after the numeric literal was migrated from `String`-buf
    /// accumulation to direct source slicing. The literal `500000` here
    /// drives the digit-accumulation branch (line 596 region); the
    /// `wallclock_ms` keeps it integer-typed (would round-trip via
    /// `SANDBOX_INT_PROPS` validation if changed). Would-FAIL pin: a
    /// regression of the slicing branch (e.g. off-by-one on `start`
    /// capture) would produce wrong integer values or a parse-error.
    #[test]
    fn issue_934_integer_literal_slice_path_round_trips() {
        let src = "handler 'h' { sandbox('m', { wallclock_ms: 12345 }) -> respond }";
        let c = compile_str(src).expect("integer literal must parse via slice path");
        assert_eq!(c.primitives.len(), 2);
        let props = &c.primitives[0].properties;
        match props.get("wallclock_ms") {
            Some(Value::Int(n)) => assert_eq!(*n, 12345, "value preserved exactly through slice"),
            other => panic!("expected Int(12345), got {other:?}"),
        }
    }

    /// #934 closure pin: float literal slice path preserves dot + digits.
    /// Exercises the `saw_dot` branch of the slicing migration. Note: floats
    /// are not currently consumed by any primitive's typed properties — the
    /// `validate_shapes` pass + the canonical surface route only Int values
    /// — so we round-trip via a transform body that flows through
    /// `parse_value` ↔ Value::Float without further validation. Would-FAIL
    /// pin: a slicing regression that dropped the `.` byte from the window
    /// would yield Int instead of Float (different variant) and fail the
    /// `matches!` assertion.
    #[test]
    fn issue_934_float_literal_slice_path_yields_float_variant() {
        let src = "handler 'h' { transform({ multiplier: 1.5 }) -> respond }";
        let c = compile_str(src).expect("float literal must parse via slice path");
        let body = c.primitives[0].properties.get("body").expect("body");
        let Value::Map(m) = body else {
            panic!("expected Map body, got {body:?}")
        };
        match m.get("multiplier") {
            Some(Value::Float(f)) if (*f - 1.5).abs() < f64::EPSILON => {}
            other => panic!("expected Float(1.5), got {other:?}"),
        }
    }

    /// #934 closure pin: negative integer literal slice path includes the
    /// leading `-` in the slice window. Would-FAIL pin: a regression that
    /// captured `start` AFTER the `if c == '-' { self.advance() }` block
    /// would slice only the digits and produce a positive value (or trip
    /// the negative-fuel `E_DSL_INVALID_SHAPE` differently).
    #[test]
    fn issue_934_negative_integer_literal_round_trips() {
        let src = "handler 'h' { transform({ delta: -42 }) -> respond }";
        let c = compile_str(src).expect("negative int must parse via slice path");
        let body = c.primitives[0].properties.get("body").expect("body");
        let Value::Map(m) = body else {
            panic!("expected Map, got {body:?}")
        };
        match m.get("delta") {
            Some(Value::Int(n)) => assert_eq!(*n, -42, "negative sign preserved via slice"),
            other => panic!("expected Int(-42), got {other:?}"),
        }
    }

    /// #934 closure pin: variable reference `$path.dotted` round-trips
    /// through the slice-path migration (cursor starts ON the `$`; the
    /// slice window captures it without a prefix `String::from("$")` boot).
    /// Would-FAIL pin: a regression that captured `start` AFTER the `$`
    /// advance would drop the leading `$` from the captured Text value and
    /// produce `path.dotted` instead of `$path.dotted`.
    #[test]
    fn issue_934_variable_reference_slice_path_preserves_dollar_prefix() {
        let src = "handler 'h' { transform({ ref: $user.id }) -> respond }";
        let c = compile_str(src).expect("var ref must parse via slice path");
        let body = c.primitives[0].properties.get("body").expect("body");
        let Value::Map(m) = body else {
            panic!("expected Map, got {body:?}")
        };
        match m.get("ref") {
            Some(Value::Text(s)) => {
                assert_eq!(s, "$user.id", "leading `$` preserved in slice window");
            }
            other => panic!("expected Text($user.id), got {other:?}"),
        }
    }

    /// #931 closure pin: the `->` chain operator dispatches correctly under
    /// the byte-comparison lookahead (the prior `peek_at(1)` UTF-8 walk was
    /// removed). Exercises the loop at line 354 (post-removal). Multi-arrow
    /// chain confirms the loop continues correctly. Would-FAIL pin: a
    /// regression of the `bytes.get(self.pos + 1)` check (e.g. wrong offset
    /// arithmetic on UTF-8-tail-byte source positions) would split the
    /// chain and yield a different primitive count or a parse-error.
    #[test]
    fn issue_931_arrow_chain_byte_comparison_dispatch() {
        // 4 primitives = 3 `->` arrows; if byte-comparison was wrong the
        // chain would terminate after the first primitive.
        let src = "handler 'h' { read('a') -> read('b') -> read('c') -> respond }";
        let c = compile_str(src).expect("3-arrow chain must parse via byte comparison");
        assert_eq!(c.primitives.len(), 4, "chain dispatched correctly");
    }

    /// #760 closure pin: `expect_char` error span points at the position
    /// the expected char SHOULD have been, not at the post-token cursor.
    /// Specifically: parse `handler 'h'` with a missing `{` — diagnostic
    /// MUST point at the column where `{` was expected. Pre-fix this would
    /// also have been roughly correct here (no token consumed past the
    /// expected position) but the helper is now centralized; this pin
    /// confirms the centralized helper still emits a non-empty span.
    #[test]
    fn issue_760_expect_char_missing_brace_carries_line_column() {
        // G-CORE-DSL chunk-3 #1000 closure: line/column now accessed via
        // the [`Diagnostic::line`] / `column` convenience accessors that
        // unwrap the underlying [`Span`].
        let src = "handler 'h'";
        let err = compile_str(src).expect_err("missing `{` must parse-error");
        let d = err.diagnostic().expect("diagnostic present");
        assert!(d.line().is_some(), "line span present");
        assert!(d.column().is_some(), "column span present");
        assert_eq!(d.error_code, E_DSL_PARSE_ERROR);
    }

    /// #760 closure pin: `unterminated string` diagnostic points at the
    /// OPENING quote's line/column rather than at the end-of-input cursor.
    /// Would-FAIL pin: pre-fix the diagnostic would carry `line:2, col:1`
    /// or similar (the post-newline cursor); post-fix it carries the
    /// quote's position on line 1.
    #[test]
    fn issue_760_unterminated_string_points_at_opening_quote() {
        // Open quote on line 1; if the parser walks to end-of-input the
        // cursor would be at line 2 (or column past where the quote opened).
        // The fix anchors the span at the opening quote.
        let src = "handler 'unterminated\nstill running";
        let err = compile_str(src).expect_err("unterminated string must parse-error");
        let d = err.diagnostic().expect("diagnostic present");
        let line = d.line().expect("line present");
        // The opening `'` of the unterminated string is on line 1 (the only
        // `'` in the source before EOF); span anchors there, not at the
        // post-walk line 2.
        assert_eq!(
            line, 1,
            "unterminated string span anchors at opening quote (line 1), not at end-of-input cursor: {d:?}"
        );
    }

    /// #760 closure pin: `expected value` diagnostic anchors at the
    /// position of the offending token rather than at the post-skip_ws
    /// cursor. The fix captures `(start_line, start_col)` immediately after
    /// `skip_ws`, BEFORE peek. (For this case both points coincide because
    /// the cursor sits on the offending char; the pin confirms a non-empty
    /// span survives the helper migration.)
    #[test]
    fn issue_760_parse_value_unexpected_token_carries_span() {
        // `]` is not a valid value-start; parser emits "expected value".
        let src = "handler 'h' { transform({ x: ] }) -> respond }";
        let err = compile_str(src).expect_err("invalid value must parse-error");
        let d = err.diagnostic().expect("diagnostic present");
        assert!(
            d.line().is_some() && d.column().is_some(),
            "span present: {d:?}"
        );
        assert_eq!(d.error_code, E_DSL_PARSE_ERROR);
    }

    /// #663 closure pin: the helper-extraction refactor preserves output
    /// for the `read` primitive (`parse_paren_string` shape).
    #[test]
    fn issue_663_read_helper_emits_identical_label() {
        let c = compile_str("handler 'h' { read('post') -> respond }").unwrap();
        assert_eq!(c.primitives[0].kind, PrimitiveKind::Read);
        match c.primitives[0].properties.get("label") {
            Some(Value::Text(s)) => assert_eq!(s, "post"),
            other => panic!("expected Text('post'), got {other:?}"),
        }
    }

    /// #663 closure pin: `parse_paren_string_optional_object` shape
    /// preserves output for `write` WITHOUT optional body — confirms the
    /// `None` arm of the helper.
    #[test]
    fn issue_663_write_helper_without_body_emits_label_only() {
        let c = compile_str("handler 'h' { write('post') -> respond }").unwrap();
        let p = &c.primitives[0];
        assert_eq!(p.kind, PrimitiveKind::Write);
        assert!(p.properties.contains_key("label"), "label present");
        assert!(
            !p.properties.contains_key("user_properties"),
            "no body emitted on `None` arm: {:?}",
            p.properties
        );
    }

    /// #663 closure pin: `parse_paren_string_optional_object` shape
    /// preserves output for `write` WITH optional body — confirms the
    /// `Some` arm of the helper.
    #[test]
    fn issue_663_write_helper_with_body_emits_user_properties() {
        let c =
            compile_str("handler 'h' { write('post', { author: 'alice' }) -> respond }").unwrap();
        let p = &c.primitives[0];
        assert_eq!(p.kind, PrimitiveKind::Write);
        assert!(p.properties.contains_key("user_properties"), "body emitted");
    }

    /// #663 closure pin: `parse_paren_object` shape preserves output for
    /// `transform` (single-object shape).
    #[test]
    fn issue_663_transform_helper_emits_body_map() {
        let c = compile_str("handler 'h' { transform({ x: 1 }) -> respond }").unwrap();
        let p = &c.primitives[0];
        assert_eq!(p.kind, PrimitiveKind::Transform);
        match p.properties.get("body") {
            Some(Value::Map(_)) => {}
            other => panic!("expected Map body, got {other:?}"),
        }
    }

    /// #663 closure pin: all 12-primitive dispatch arms still surface
    /// after the helper refactor. Exercises read / write / transform /
    /// branch / iterate / wait / call / sandbox / emit / subscribe /
    /// stream / respond in a single round-trip + a perm-equivalence pin
    /// over canonical bytes via the cross-surface
    /// `permuted_keys_yield_identical_canonical_bytes` (preserves the
    /// upstream pin's coverage at the helper-refactor boundary).
    #[test]
    fn issue_663_all_12_primitive_kinds_dispatch_via_helpers() {
        // Single-primitive smokes (separate compiles so each handler is
        // standalone + has a `respond`).
        let cases = [
            ("read", "handler 'h' { read('x') -> respond }"),
            ("write", "handler 'h' { write('x') -> respond }"),
            (
                "transform",
                "handler 'h' { transform({ y: 1 }) -> respond }",
            ),
            ("branch", "handler 'h' { branch(true) -> respond }"),
            ("iterate", "handler 'h' { iterate(3) -> respond }"),
            (
                "wait",
                "handler 'h' { wait({ duration_ms: 100 }) -> respond }",
            ),
            ("call", "handler 'h' { call('other') -> respond }"),
            ("sandbox", "handler 'h' { sandbox('m') -> respond }"),
            ("emit", "handler 'h' { emit('topic') -> respond }"),
            ("subscribe", "handler 'h' { subscribe('topic') -> respond }"),
            ("stream", "handler 'h' { stream('chunks') -> respond }"),
            ("respond", "handler 'h' { respond }"),
        ];
        for (name, src) in cases {
            let c = compile_str(src).unwrap_or_else(|e| panic!("{name} dispatch failed: {e:?}"));
            assert!(
                c.primitives.iter().any(|p| matches!(
                    (p.kind, name),
                    (PrimitiveKind::Read, "read")
                        | (PrimitiveKind::Write, "write")
                        | (PrimitiveKind::Transform, "transform")
                        | (PrimitiveKind::Branch, "branch")
                        | (PrimitiveKind::Iterate, "iterate")
                        | (PrimitiveKind::Wait, "wait")
                        | (PrimitiveKind::Call, "call")
                        | (PrimitiveKind::Sandbox, "sandbox")
                        | (PrimitiveKind::Emit, "emit")
                        | (PrimitiveKind::Subscribe, "subscribe")
                        | (PrimitiveKind::Stream, "stream")
                        | (PrimitiveKind::Respond, "respond")
                )),
                "{name} primitive landed in the compiled set: {:?}",
                c.primitives.iter().map(|p| p.kind).collect::<Vec<_>>()
            );
        }
    }
}
