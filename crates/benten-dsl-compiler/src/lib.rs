//! `benten-dsl-compiler` — Phase-2b DSL-text → `Subgraph` compiler.
//!
//! **MINIMAL-FOR-DEVSERVER scope** per `r1-architect-reviewer.json` (G12-B-scope):
//! ~900 LOC, 7 public items intended for `tools/benten-dev` (the literal LOC
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

/// Compile a DSL source string into a [`CompiledSubgraph`].
///
/// # Errors
///
/// Returns [`CompileError`] for any parse, semantic, or emission failure.
/// Each error carries a [`Diagnostic`] with line/column + human-readable
/// message + typed `error_code` for devserver rendering.
pub fn compile_str(source: &str) -> Result<CompiledSubgraph, CompileError> {
    if source.trim().is_empty() {
        return Err(CompileError::Parse(Diagnostic {
            error_code: E_DSL_PARSE_ERROR,
            message: "empty DSL source".to_string(),
            line: None,
            column: None,
        }));
    }
    let mut parser = Parser::new(source);
    let handler = parser.parse_handler()?;
    emit(handler)
}

/// Compile a DSL source file into a [`CompiledSubgraph`].
///
/// # Errors
///
/// Returns [`CompileError::Io`] for IO failures or any failure modes of
/// [`compile_str`].
pub fn compile_file(path: &Path) -> Result<CompiledSubgraph, CompileError> {
    let src = std::fs::read_to_string(path)
        .map_err(|e| CompileError::Io(format!("{}: {}", path.display(), e)))?;
    compile_str(&src)
}

/// Typed compile-error enum surfaced to devserver + downstream tools.
///
/// Wire-stable variant set: each variant maps to a stable `error_code`
/// string (see [`Diagnostic::error_code`]) so devserver / TS-side renderers
/// can switch on the discriminant without prose-string parsing.
#[derive(Debug, Clone, Error)]
pub enum CompileError {
    /// Lexer / parser failure — DSL did not match the grammar.
    #[error("DSL parse error: {0}")]
    Parse(Diagnostic),
    /// Semantic / type-check failure — DSL parsed but referenced unknown
    /// primitives / props / handler ids.
    #[error("DSL semantic error: {0}")]
    Semantic(Diagnostic),
    /// Emission failure — well-typed AST but `Subgraph` construction
    /// rejected (e.g. missing RESPOND, malformed structural shape).
    #[error("DSL emit error: {0}")]
    Emit(Diagnostic),
    /// IO failure reading a source file (only from [`compile_file`]).
    #[error("DSL io error: {0}")]
    Io(String),
}

impl CompileError {
    /// Borrow the inner [`Diagnostic`] when present.
    #[must_use]
    pub fn diagnostic(&self) -> Option<&Diagnostic> {
        match self {
            Self::Parse(d) | Self::Semantic(d) | Self::Emit(d) => Some(d),
            Self::Io(_) => None,
        }
    }
}

/// Diagnostic shape devserver renders: span + message + typed error code.
#[derive(Debug, Clone)]
pub struct Diagnostic {
    /// Stable error-code string (e.g. `"E_DSL_PARSE_ERROR"`); switch-keyed
    /// by devserver renderer.
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

// ---------------------------------------------------------------------------
// Stable error codes
// ---------------------------------------------------------------------------

pub(crate) const E_DSL_PARSE_ERROR: &str = "E_DSL_PARSE_ERROR";
pub(crate) const E_DSL_UNKNOWN_PRIMITIVE: &str = "E_DSL_UNKNOWN_PRIMITIVE";
pub(crate) const E_DSL_MISSING_RESPOND: &str = "E_DSL_MISSING_RESPOND";
/// Phase-3 R6 fp Wave C2 (closes dx-r6-r1-1 MAJOR — DSL orphan code half):
/// shape validation rejected a primitive's typed property (e.g. SANDBOX
/// `fuel` declared as a string instead of an integer). Mirrors the
/// TS-side `EDslInvalidShape` thrown from `packages/engine/src/dsl.ts`
/// builder methods so a Rust callsite emitting this surfaces the same
/// typed `BentenError` subclass on the wire. Drift-detect reachability
/// path: `crates/benten-dsl-compiler/src/lib.rs::validate_shapes` (a
/// crate-private free function, NOT a member of an `emit` module —
/// `emit` and `validate_shapes` are sibling free functions).
pub(crate) const E_DSL_INVALID_SHAPE: &str = "E_DSL_INVALID_SHAPE";

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
            if self.peek() == Some('-') && self.peek_at(1) == Some('>') {
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

    #[allow(
        clippy::too_many_lines,
        reason = "single dispatch table for the 12 primitive keywords; \
                  splitting per-kind helpers would just spread the same \
                  match arm across 12 single-call functions"
    )]
    fn parse_primitive(&mut self) -> Result<PrimitiveAst, CompileError> {
        let (start_line, start_col) = (self.line, self.column);
        let ident = self.parse_identifier()?;
        let mut props = std::collections::BTreeMap::<String, Value>::new();
        let kind = match ident.as_str() {
            "read" => {
                self.skip_ws();
                self.expect_char('(')?;
                self.skip_ws();
                let label = self.parse_string()?;
                self.skip_ws();
                self.expect_char(')')?;
                props.insert(KEY_LABEL.to_string(), Value::Text(label));
                PrimitiveKind::Read
            }
            "write" => {
                self.skip_ws();
                self.expect_char('(')?;
                self.skip_ws();
                let label = self.parse_string()?;
                props.insert(KEY_LABEL.to_string(), Value::Text(label));
                self.skip_ws();
                if self.peek() == Some(',') {
                    self.advance();
                    self.skip_ws();
                    let body = self.parse_object()?;
                    props.insert(KEY_USER_PROPERTIES.to_string(), Value::Map(body));
                }
                self.skip_ws();
                self.expect_char(')')?;
                PrimitiveKind::Write
            }
            "transform" => {
                self.skip_ws();
                self.expect_char('(')?;
                self.skip_ws();
                let body = self.parse_object()?;
                props.insert(KEY_TRANSFORM_BODY.to_string(), Value::Map(body));
                self.skip_ws();
                self.expect_char(')')?;
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
                self.skip_ws();
                self.expect_char('(')?;
                self.skip_ws();
                let body = self.parse_object()?;
                for (k, v) in body {
                    props.insert(k, v);
                }
                self.skip_ws();
                self.expect_char(')')?;
                PrimitiveKind::Wait
            }
            "call" => {
                self.skip_ws();
                self.expect_char('(')?;
                self.skip_ws();
                let target = self.parse_string()?;
                props.insert(KEY_CALL_HANDLER.to_string(), Value::Text(target));
                self.skip_ws();
                if self.peek() == Some(',') {
                    self.advance();
                    self.skip_ws();
                    let body = self.parse_object()?;
                    props.insert(KEY_CALL_ARGS.to_string(), Value::Map(body));
                }
                self.skip_ws();
                self.expect_char(')')?;
                PrimitiveKind::Call
            }
            "sandbox" => {
                self.skip_ws();
                self.expect_char('(')?;
                self.skip_ws();
                let module = self.parse_string()?;
                props.insert(KEY_SANDBOX_MODULE.to_string(), Value::Text(module));
                self.skip_ws();
                if self.peek() == Some(',') {
                    self.advance();
                    self.skip_ws();
                    let body = self.parse_object()?;
                    for (k, v) in body {
                        props.insert(k, v);
                    }
                }
                self.skip_ws();
                self.expect_char(')')?;
                PrimitiveKind::Sandbox
            }
            "respond" => PrimitiveKind::Respond,
            "emit" => {
                self.skip_ws();
                self.expect_char('(')?;
                self.skip_ws();
                let topic = self.parse_string()?;
                props.insert(KEY_EMIT_TOPIC.to_string(), Value::Text(topic));
                self.skip_ws();
                self.expect_char(')')?;
                PrimitiveKind::Emit
            }
            "subscribe" => {
                self.skip_ws();
                self.expect_char('(')?;
                self.skip_ws();
                let pattern = self.parse_string()?;
                props.insert(KEY_SUBSCRIBE_PATTERN.to_string(), Value::Text(pattern));
                self.skip_ws();
                self.expect_char(')')?;
                PrimitiveKind::Subscribe
            }
            "stream" => {
                self.skip_ws();
                self.expect_char('(')?;
                self.skip_ws();
                let label = self.parse_string()?;
                props.insert(KEY_LABEL.to_string(), Value::Text(label));
                self.skip_ws();
                self.expect_char(')')?;
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
                return Err(CompileError::Semantic(Diagnostic {
                    error_code: E_DSL_UNKNOWN_PRIMITIVE,
                    message: format!("unknown primitive `{other}`"),
                    line: Some(start_line),
                    column: Some(start_col),
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
            match self.peek() {
                Some(',') => {
                    self.advance();
                }
                Some('}') => {
                    self.advance();
                    break;
                }
                _ => return Err(self.parse_err("expected ',' or '}'".to_string())),
            }
        }
        Ok(map)
    }

    fn parse_value(&mut self) -> Result<Value, CompileError> {
        self.skip_ws();
        match self.peek() {
            Some('\'') => Ok(Value::Text(self.parse_string()?)),
            Some('{') => Ok(Value::Map(self.parse_object()?)),
            Some('$') => {
                // Variable reference — preserve as `$path.dotted`.
                self.advance();
                let mut buf = String::from("$");
                while let Some(c) = self.peek() {
                    if c.is_ascii_alphanumeric() || c == '_' || c == '.' {
                        buf.push(c);
                        self.advance();
                    } else {
                        break;
                    }
                }
                Ok(Value::Text(buf))
            }
            Some(c) if c.is_ascii_digit() || c == '-' => {
                let mut buf = String::new();
                if c == '-' {
                    buf.push('-');
                    self.advance();
                }
                let mut saw_dot = false;
                while let Some(c) = self.peek() {
                    if c.is_ascii_digit() {
                        buf.push(c);
                        self.advance();
                    } else if c == '.' && !saw_dot {
                        saw_dot = true;
                        buf.push(c);
                        self.advance();
                    } else {
                        break;
                    }
                }
                if saw_dot {
                    let v: f64 = buf
                        .parse()
                        .map_err(|_| self.parse_err(format!("invalid float `{buf}`")))?;
                    Ok(Value::Float(v))
                } else {
                    let v: i64 = buf
                        .parse()
                        .map_err(|_| self.parse_err(format!("invalid int `{buf}`")))?;
                    Ok(Value::Int(v))
                }
            }
            Some('t' | 'f') => {
                let ident = self.parse_identifier()?;
                match ident.as_str() {
                    "true" => Ok(Value::Bool(true)),
                    "false" => Ok(Value::Bool(false)),
                    other => Err(self.parse_err(format!("unexpected identifier `{other}`"))),
                }
            }
            _ => Err(self.parse_err("expected value".to_string())),
        }
    }

    // -- token helpers --

    fn parse_string(&mut self) -> Result<String, CompileError> {
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
                None => return Err(self.parse_err("unterminated string".to_string())),
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
            return Err(CompileError::Parse(Diagnostic {
                error_code: E_DSL_PARSE_ERROR,
                message: "expected identifier".to_string(),
                line: Some(start_line),
                column: Some(start_col),
            }));
        }
        Ok(self.src[start..self.pos].to_string())
    }

    fn expect_keyword(&mut self, kw: &str) -> Result<(), CompileError> {
        let id = self.parse_identifier()?;
        if id == kw {
            Ok(())
        } else {
            Err(self.parse_err(format!("expected keyword `{kw}`, got `{id}`")))
        }
    }

    fn expect_char(&mut self, c: char) -> Result<(), CompileError> {
        match self.peek() {
            Some(p) if p == c => {
                self.advance();
                Ok(())
            }
            Some(p) => Err(self.parse_err(format!("expected `{c}`, got `{p}`"))),
            None => Err(self.parse_err(format!("expected `{c}`, got end-of-input"))),
        }
    }

    fn read_until_balanced(&mut self, close: char) -> Result<String, CompileError> {
        // Treat `(` / `)` parens balancing for a `branch(...)` or
        // `iterate(...)` expression body. Handles nested parens; stops at
        // the unbalanced `close` char without consuming it.
        let start = self.pos;
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
                None => return Err(self.parse_err(format!("expected `{close}`, hit end-of-input"))),
            }
        }
        Ok(self.src[start..self.pos].to_string())
    }

    fn parse_err(&self, message: String) -> CompileError {
        CompileError::Parse(Diagnostic {
            error_code: E_DSL_PARSE_ERROR,
            message,
            line: Some(self.line),
            column: Some(self.column),
        })
    }

    // -- low-level cursor --

    fn peek(&self) -> Option<char> {
        self.src[self.pos..].chars().next()
    }

    fn peek_at(&self, n: usize) -> Option<char> {
        self.src[self.pos..].chars().nth(n)
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
// Emit — AST → benten_core::Subgraph + CompiledPrimitive list.
// ---------------------------------------------------------------------------

fn emit(handler: HandlerAst) -> Result<CompiledSubgraph, CompileError> {
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
        return Err(CompileError::Emit(Diagnostic {
            error_code: E_DSL_MISSING_RESPOND,
            message: format!(
                "handler `{}` does not contain a `respond` primitive",
                handler.handler_id
            ),
            line: None,
            column: None,
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

    for p in &handler.primitives {
        if matches!(p.kind, PrimitiveKind::Sandbox) {
            for &key in SANDBOX_INT_PROPS {
                if let Some(v) = p.properties.get(key) {
                    match v {
                        Value::Int(n) if *n >= 0 => {}
                        Value::Int(n) => {
                            return Err(CompileError::Emit(Diagnostic {
                                error_code: E_DSL_INVALID_SHAPE,
                                message: format!(
                                    "sandbox primitive `{}` property must be a non-negative integer (got {n}); see docs/SANDBOX-LIMITS.md §2",
                                    key
                                ),
                                line: None,
                                column: None,
                            }));
                        }
                        other => {
                            return Err(CompileError::Emit(Diagnostic {
                                error_code: E_DSL_INVALID_SHAPE,
                                message: format!(
                                    "sandbox primitive `{}` property must be a non-negative integer (got {:?}); see docs/SANDBOX-LIMITS.md §2",
                                    key, other
                                ),
                                line: None,
                                column: None,
                            }));
                        }
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
        // PrimitiveKind is `#[non_exhaustive]`. New variants added in later
        // phases fall back to a generic `op` prefix; the DSL grammar does
        // not yet have keywords for them, so this branch is unreachable
        // from the parser today but keeps the compile honest.
        _ => "op",
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
    fn missing_respond_is_typed_emit_error() {
        let err = compile_str("handler 'h' { read('post') }").unwrap_err();
        assert!(matches!(err, CompileError::Emit(_)));
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
        let src = "handler 'h' { sandbox('mod', { fuel: 'high' }) -> respond }";
        let err = compile_str(src).unwrap_err();
        assert!(
            matches!(err, CompileError::Emit(_)),
            "fuel-as-string trips Emit-shape error, got {err:?}"
        );
        assert_eq!(
            err.diagnostic().unwrap().error_code,
            E_DSL_INVALID_SHAPE,
            "must surface E_DSL_INVALID_SHAPE typed code"
        );
    }

    #[test]
    fn sandbox_negative_fuel_is_typed_invalid_shape() {
        let src = "handler 'h' { sandbox('mod', { fuel: -1 }) -> respond }";
        let err = compile_str(src).unwrap_err();
        assert!(
            matches!(err, CompileError::Emit(_)),
            "negative fuel trips Emit-shape error, got {err:?}"
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
}
