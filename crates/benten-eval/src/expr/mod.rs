//! TRANSFORM expression language implementation (G6-B).
//!
//! Implements the positive-allowlist grammar documented in
//! `docs/TRANSFORM-GRAMMAR.md`. Three sibling modules:
//!
//! - [`parser`] — hand-rolled Pratt-style recursive descent parser. Rejects
//!   any construct not in the BNF with `E_TRANSFORM_SYNTAX` carrying the
//!   byte offset of the first rejected token.
//! - [`eval`] — pure deterministic evaluator. Walks a parsed [`Expr`]
//!   against an [`eval::Env`] binding frame stack. No engine access, no
//!   clock, no I/O, no RNG.
//! - [`builtins`] — the 50+ built-in call dispatchers (arithmetic, string,
//!   array, object, coercion, number formatting).
//!
//! The public entry points re-exported from `benten_eval::transform` call
//! into this module.

pub mod builtins;
pub mod eval;
pub mod parser;

use benten_core::Value;
use std::collections::BTreeMap;

/// Parsed TRANSFORM expression — the allowlist-only AST.
///
/// Every variant is positively admitted by the BNF in
/// `docs/TRANSFORM-GRAMMAR.md`. The parser constructs this AST; no other
/// shape is producible. The allowlist-only property is structurally
/// enforced by the types.
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    /// Literal value (number / string / bool / null).
    Literal(Value),
    /// Bare identifier — resolved against the evaluation context.
    Identifier(String),
    /// Context binding (`$input`, `$result`, `$item`, `$index`, `$results`, `$error`).
    ContextBinding(String),
    /// Binary operator (`+`, `-`, `*`, `/`, `%`, `<`, `<=`, `>`, `>=`, `==`, `!=`, `===`, `!==`, `&&`, `||`).
    Binary {
        op: BinaryOp,
        lhs: Box<Expr>,
        rhs: Box<Expr>,
    },
    /// Unary operator (`!`, `-`, `+`).
    Unary { op: UnaryOp, expr: Box<Expr> },
    /// Conditional expression (ternary).
    Conditional {
        cond: Box<Expr>,
        then_branch: Box<Expr>,
        else_branch: Box<Expr>,
    },
    /// Property access: `obj.name`.
    PropertyAccess { target: Box<Expr>, name: String },
    /// Index access: `obj["k"]` / `arr[0]`.
    IndexAccess { target: Box<Expr>, index: Box<Expr> },
    /// Invocation (built-in call or method call).
    Call { callee: Box<Expr>, args: Vec<Expr> },
    /// Array literal.
    Array(Vec<Expr>),
    /// Object literal.
    Object(Vec<(String, Expr)>),
    /// Lambda expression — only valid as an argument to specific array
    /// methods (`map`, `filter`, `reduce`, `find`, `findIndex`, `every`,
    /// `some`). The parser admits lambdas only in those positions; bare
    /// lambdas in expression position are rejected with `E_TRANSFORM_SYNTAX`.
    Lambda {
        params: Vec<String>,
        body: Box<Expr>,
    },
}

/// Binary operators admitted by the grammar.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Lt,
    Le,
    Gt,
    Ge,
    Eq,
    Ne,
    EqStrict,
    NeStrict,
    And,
    Or,
}

/// Unary operators admitted by the grammar.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOp {
    Not,
    Neg,
    Pos,
}

impl Expr {
    /// Structural allowlist check.
    ///
    /// Returns `true` if every node in the tree is one of the documented
    /// allowlisted variants. Since the parser exclusively produces these
    /// variants, this check is vacuously true for any AST constructed by
    /// [`parser::parse`]. The function exists to satisfy the fuzz-harness
    /// contract in `crates/benten-eval/tests/transform_grammar_fuzz.rs`.
    #[must_use]
    pub fn uses_only_allowlisted_nodes(&self) -> bool {
        match self {
            Expr::Literal(_) | Expr::Identifier(_) | Expr::ContextBinding(_) => true,
            Expr::Binary { lhs, rhs, .. } => {
                lhs.uses_only_allowlisted_nodes() && rhs.uses_only_allowlisted_nodes()
            }
            Expr::Unary { expr, .. } => expr.uses_only_allowlisted_nodes(),
            Expr::Conditional {
                cond,
                then_branch,
                else_branch,
            } => {
                cond.uses_only_allowlisted_nodes()
                    && then_branch.uses_only_allowlisted_nodes()
                    && else_branch.uses_only_allowlisted_nodes()
            }
            Expr::PropertyAccess { target, .. } => target.uses_only_allowlisted_nodes(),
            Expr::IndexAccess { target, index } => {
                target.uses_only_allowlisted_nodes() && index.uses_only_allowlisted_nodes()
            }
            Expr::Call { callee, args } => {
                callee.uses_only_allowlisted_nodes()
                    && args.iter().all(Self::uses_only_allowlisted_nodes)
            }
            Expr::Array(items) => items.iter().all(Self::uses_only_allowlisted_nodes),
            Expr::Object(fields) => fields.iter().all(|(_, v)| v.uses_only_allowlisted_nodes()),
            Expr::Lambda { body, .. } => body.uses_only_allowlisted_nodes(),
        }
    }
}

/// Diagnostic wrapper built from a [`BTreeMap`] payload; used by the
/// evaluator's error-routing surface when a built-in call fails.
#[must_use]
pub fn make_map(pairs: Vec<(String, Value)>) -> Value {
    let mut m = BTreeMap::new();
    for (k, v) in pairs {
        m.insert(k, v);
    }
    Value::Map(m)
}
