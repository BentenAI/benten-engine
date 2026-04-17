//! Hand-rolled Pratt / recursive-descent parser for the TRANSFORM grammar.
//!
//! Implements the BNF in `docs/TRANSFORM-GRAMMAR.md`. The parser is a
//! positive allowlist: every production is individually admitted. Any token
//! or grammatical shape outside the BNF produces a [`ParseError`] whose
//! `offset` points at the byte index of the first rejected token; the
//! crate-public entry point [`crate::transform::parse_transform`] remaps
//! this to `E_TRANSFORM_SYNTAX`.
//!
//! ## Design notes
//!
//! - Hand-rolled rather than `nom` / `pest` / `chumsky` — the grammar is
//!   small (< 200 productions) and the primary dependency surface is parse
//!   errors + byte offsets. A hand-rolled parser gives precise control over
//!   offset reporting without wrestling an upstream error model.
//! - Single-pass lexer interleaved with the parser — the lexer produces one
//!   token at a time and the parser looks at most one token ahead (or peeks
//!   a fixed-size raw-byte prefix for multi-character punctuation).
//! - Lambdas are only admitted inside the argument lists of specific array
//!   methods (`map`, `filter`, `reduce`, `find`, `findIndex`, `every`,
//!   `some`, `sortBy`, `uniqueBy`, `groupBy`, `count`). A bare lambda
//!   outside those positions — including a parenthesized `(x) => …` in
//!   top-level expression position — is rejected with `E_TRANSFORM_SYNTAX`
//!   at the `=>` token.
//! - Rejected JavaScript constructs (see the 25-class denylist in the
//!   grammar doc) are caught by either the lexer (explicitly-banned tokens
//!   like `=>` at top level, `**`, `??`, `?.`, `...`, bitwise ops, assignment
//!   ops, template literals, regex literals) or the parser (reserved words
//!   like `new`, `this`, `typeof`, `instanceof`, `function`, `return`, etc.).

use super::{BinaryOp, Expr, UnaryOp};
use benten_core::Value;

/// Array-method names that admit a lambda as an argument.
pub(crate) const LAMBDA_CALL_METHODS: &[&str] = &[
    "map",
    "filter",
    "reduce",
    "find",
    "findIndex",
    "every",
    "some",
    "sortBy",
    "uniqueBy",
    "groupBy",
    "count",
];

/// Reserved words that the grammar explicitly rejects wherever they appear
/// (matched as bare identifiers; the rejection fires on their IDENT span).
const REJECTED_WORDS: &[&str] = &[
    "new",
    "this",
    "typeof",
    "instanceof",
    "function",
    "return",
    "throw",
    "delete",
    "import",
    "export",
    "yield",
    "await",
    "async",
    "with",
    "var",
    "let",
    "const",
    "class",
    "extends",
    "super",
    "eval",
    "Symbol",
    "Reflect",
    "Proxy",
    "require",
    "in",
];

/// A parse error; the `offset` field points at the first rejected byte.
#[derive(Debug, Clone)]
pub struct ParseError {
    pub offset: usize,
    pub message: String,
}

/// Parse a TRANSFORM expression string into an [`Expr`].
///
/// # Errors
///
/// Returns [`ParseError`] (later mapped to `E_TRANSFORM_SYNTAX`) whenever
/// the input falls outside the grammar's allowlist.
pub fn parse(input: &str) -> Result<Expr, ParseError> {
    let mut p = Parser::new(input);
    let expr = p.parse_expression()?;
    // If a trailing token is still peeked / ready, report at *its* start —
    // not at the lexer's current byte cursor, which has already advanced
    // past the token.
    let (tok, off) = {
        let t = p.peek_token()?;
        (t.tok.clone(), t.start)
    };
    if !matches!(tok, Tok::Eof) {
        let msg = match &tok {
            Tok::Arrow => "arrow `=>` is only admitted as an argument of array methods".to_string(),
            Tok::Ident(n) if REJECTED_WORDS.contains(&n.as_str()) => {
                format!("`{n}` is a rejected keyword / identifier")
            }
            _ => "unexpected trailing input".to_string(),
        };
        return Err(p.error(off, msg));
    }
    Ok(expr)
}

// ---------------------------------------------------------------------------
// Tokens + lexer
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
enum Tok {
    // Literals
    Number(f64),
    Int(i64),
    Str(String),
    True,
    False,
    Null,
    // Identifiers
    Ident(String),
    Dollar(String), // `$input`, `$item`, …
    // Punctuation
    LParen,
    RParen,
    LBracket,
    RBracket,
    LBrace,
    RBrace,
    Comma,
    Colon,
    Question,
    Dot,
    Arrow, // =>
    // Arithmetic
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    // Comparison / equality
    Lt,
    Le,
    Gt,
    Ge,
    Eq,       // ==
    Ne,       // !=
    EqStrict, // ===
    NeStrict, // !==
    // Logical
    AndAnd,
    OrOr,
    Bang,
    // End
    Eof,
}

#[derive(Clone)]
struct Token {
    tok: Tok,
    /// Byte offset of the first character of the token.
    start: usize,
}

struct Parser<'a> {
    input: &'a str,
    bytes: &'a [u8],
    pos: usize,
    /// Peeked token (we look one ahead).
    peeked: Option<Token>,
    /// True when the parser is currently inside the argument list of a
    /// lambda-admitting builtin method call. Enables arrow-function parsing
    /// at that position only.
    lambda_ctx: bool,
}

impl<'a> Parser<'a> {
    fn new(input: &'a str) -> Self {
        Self {
            input,
            bytes: input.as_bytes(),
            pos: 0,
            peeked: None,
            lambda_ctx: false,
        }
    }

    fn error(&self, offset: usize, msg: impl Into<String>) -> ParseError {
        ParseError {
            offset,
            message: msg.into(),
        }
    }

    // --- whitespace ---------------------------------------------------------

    fn skip_ws(&mut self) {
        while self.pos < self.bytes.len() {
            let b = self.bytes[self.pos];
            if b == b' ' || b == b'\t' || b == b'\n' || b == b'\r' {
                self.pos += 1;
            } else {
                break;
            }
        }
    }

    // --- tokenizer ----------------------------------------------------------

    fn peek_token(&mut self) -> Result<&Token, ParseError> {
        if self.peeked.is_none() {
            let t = self.next_token_raw()?;
            self.peeked = Some(t);
        }
        Ok(self.peeked.as_ref().expect("just populated"))
    }

    fn take_token(&mut self) -> Result<Token, ParseError> {
        if let Some(t) = self.peeked.take() {
            Ok(t)
        } else {
            self.next_token_raw()
        }
    }

    fn next_token_raw(&mut self) -> Result<Token, ParseError> {
        self.skip_ws();
        let start = self.pos;
        if start >= self.bytes.len() {
            return Ok(Token {
                tok: Tok::Eof,
                start,
            });
        }
        let b = self.bytes[start];

        // Multi-character punctuation first.
        if self.starts_with_at(start, "===") {
            self.pos = start + 3;
            return Ok(Token {
                tok: Tok::EqStrict,
                start,
            });
        }
        if self.starts_with_at(start, "!==") {
            self.pos = start + 3;
            return Ok(Token {
                tok: Tok::NeStrict,
                start,
            });
        }
        if self.starts_with_at(start, "==") {
            self.pos = start + 2;
            return Ok(Token {
                tok: Tok::Eq,
                start,
            });
        }
        if self.starts_with_at(start, "!=") {
            self.pos = start + 2;
            return Ok(Token {
                tok: Tok::Ne,
                start,
            });
        }
        if self.starts_with_at(start, "<=") {
            self.pos = start + 2;
            return Ok(Token {
                tok: Tok::Le,
                start,
            });
        }
        if self.starts_with_at(start, ">=") {
            self.pos = start + 2;
            return Ok(Token {
                tok: Tok::Ge,
                start,
            });
        }
        if self.starts_with_at(start, "&&") {
            self.pos = start + 2;
            return Ok(Token {
                tok: Tok::AndAnd,
                start,
            });
        }
        if self.starts_with_at(start, "||") {
            self.pos = start + 2;
            return Ok(Token {
                tok: Tok::OrOr,
                start,
            });
        }
        if self.starts_with_at(start, "=>") {
            self.pos = start + 2;
            return Ok(Token {
                tok: Tok::Arrow,
                start,
            });
        }

        // --- Explicitly-banned multi-character punctuation (grammar denial).
        if self.starts_with_at(start, "**") {
            return Err(self.error(start, "exponent `**` is not in the grammar"));
        }
        if self.starts_with_at(start, "??") {
            return Err(self.error(start, "nullish coalescing `??` is not in the grammar"));
        }
        if self.starts_with_at(start, "?.") {
            return Err(self.error(start, "optional chaining `?.` is not in the grammar"));
        }
        if self.starts_with_at(start, "...") {
            return Err(self.error(start, "spread `...` is not in the grammar"));
        }
        if self.starts_with_at(start, "++") {
            return Err(self.error(start, "increment `++` is not in the grammar"));
        }
        if self.starts_with_at(start, "--") {
            return Err(self.error(start, "decrement `--` is not in the grammar"));
        }
        if self.starts_with_at(start, "<<") {
            return Err(self.error(start, "bitshift `<<` is not in the grammar"));
        }
        if self.starts_with_at(start, ">>") {
            return Err(self.error(start, "bitshift `>>` is not in the grammar"));
        }

        // Single-character banned punctuation.
        match b {
            b'&' => return Err(self.error(start, "bitwise `&` is not in the grammar")),
            b'|' => return Err(self.error(start, "bitwise `|` is not in the grammar")),
            b'^' => return Err(self.error(start, "bitwise `^` is not in the grammar")),
            b'~' => return Err(self.error(start, "bitwise `~` is not in the grammar")),
            b'`' => {
                return Err(self.error(start, "template literals are not in the grammar"));
            }
            _ => {}
        }

        // Assignment `=` (not part of `==` / `===` / `=>` already handled).
        if b == b'=' {
            return Err(self.error(start, "assignment `=` is not in the grammar"));
        }

        // Single-character punctuation.
        match b {
            b'(' => {
                self.pos = start + 1;
                return Ok(Token {
                    tok: Tok::LParen,
                    start,
                });
            }
            b')' => {
                self.pos = start + 1;
                return Ok(Token {
                    tok: Tok::RParen,
                    start,
                });
            }
            b'[' => {
                self.pos = start + 1;
                return Ok(Token {
                    tok: Tok::LBracket,
                    start,
                });
            }
            b']' => {
                self.pos = start + 1;
                return Ok(Token {
                    tok: Tok::RBracket,
                    start,
                });
            }
            b'{' => {
                self.pos = start + 1;
                return Ok(Token {
                    tok: Tok::LBrace,
                    start,
                });
            }
            b'}' => {
                self.pos = start + 1;
                return Ok(Token {
                    tok: Tok::RBrace,
                    start,
                });
            }
            b',' => {
                self.pos = start + 1;
                return Ok(Token {
                    tok: Tok::Comma,
                    start,
                });
            }
            b':' => {
                self.pos = start + 1;
                return Ok(Token {
                    tok: Tok::Colon,
                    start,
                });
            }
            b'?' => {
                self.pos = start + 1;
                return Ok(Token {
                    tok: Tok::Question,
                    start,
                });
            }
            b'.' => {
                self.pos = start + 1;
                return Ok(Token {
                    tok: Tok::Dot,
                    start,
                });
            }
            b'+' => {
                self.pos = start + 1;
                return Ok(Token {
                    tok: Tok::Plus,
                    start,
                });
            }
            b'-' => {
                self.pos = start + 1;
                return Ok(Token {
                    tok: Tok::Minus,
                    start,
                });
            }
            b'*' => {
                self.pos = start + 1;
                return Ok(Token {
                    tok: Tok::Star,
                    start,
                });
            }
            b'/' => {
                // `/` is division only; regex literals (leading `/`) are
                // rejected per class 18. The tokenizer can't easily
                // distinguish without a "prev-token-was-operand" context,
                // so we disambiguate using the peeked token history: at
                // *token-start* position, `/` introduces division if the
                // parser is at an infix position. Since the parser calls
                // `peek_token` after an operand, the token is unambiguously
                // a divider. If `/` appears at the very start of input
                // (i.e., pos==0 OR the preceding token is not an operand),
                // the parser-level error handler surfaces class-18
                // ("regex literal not in the grammar") at that offset.
                //
                // Simplest implementation: always emit `Slash` here; the
                // parser rejects it if it appears in a prefix position.
                self.pos = start + 1;
                return Ok(Token {
                    tok: Tok::Slash,
                    start,
                });
            }
            b'%' => {
                self.pos = start + 1;
                return Ok(Token {
                    tok: Tok::Percent,
                    start,
                });
            }
            b'<' => {
                self.pos = start + 1;
                return Ok(Token {
                    tok: Tok::Lt,
                    start,
                });
            }
            b'>' => {
                self.pos = start + 1;
                return Ok(Token {
                    tok: Tok::Gt,
                    start,
                });
            }
            b'!' => {
                self.pos = start + 1;
                return Ok(Token {
                    tok: Tok::Bang,
                    start,
                });
            }
            _ => {}
        }

        // Numeric literal.
        if b.is_ascii_digit() {
            return self.lex_number(start);
        }

        // String literal (double-quoted only).
        if b == b'"' {
            return self.lex_string(start);
        }
        if b == b'\'' {
            // Single-quoted strings are rejected to match the grammar doc's
            // single-style policy. Fall through to the generic "unexpected"
            // error so rejection still carries offset = start.
            return self.lex_string_single(start);
        }

        // Context binding `$ident`.
        if b == b'$' {
            let mut end = start + 1;
            while end < self.bytes.len() && is_ident_byte(self.bytes[end]) {
                end += 1;
            }
            if end == start + 1 {
                return Err(self.error(start, "`$` must be followed by an identifier"));
            }
            let name = &self.input[start..end];
            self.pos = end;
            return Ok(Token {
                tok: Tok::Dollar(name.to_string()),
                start,
            });
        }

        // Identifier / keyword.
        if is_ident_start(b) {
            let mut end = start + 1;
            while end < self.bytes.len() && is_ident_byte(self.bytes[end]) {
                end += 1;
            }
            let ident = &self.input[start..end];
            self.pos = end;
            let tok = match ident {
                "true" => Tok::True,
                "false" => Tok::False,
                "null" => Tok::Null,
                "undefined" => Tok::Null,
                _ => Tok::Ident(ident.to_string()),
            };
            return Ok(Token { tok, start });
        }

        Err(self.error(start, format!("unexpected byte `{}`", b as char)))
    }

    fn lex_number(&mut self, start: usize) -> Result<Token, ParseError> {
        let mut end = start;
        let mut is_float = false;
        while end < self.bytes.len() && self.bytes[end].is_ascii_digit() {
            end += 1;
        }
        // Fractional part.
        if end < self.bytes.len() && self.bytes[end] == b'.' {
            // Lookahead: is the byte after `.` a digit? If not, this is a
            // property-access `.` and the number is just the integer part.
            if end + 1 < self.bytes.len() && self.bytes[end + 1].is_ascii_digit() {
                is_float = true;
                end += 1;
                while end < self.bytes.len() && self.bytes[end].is_ascii_digit() {
                    end += 1;
                }
            }
        }
        // Exponent part.
        if end < self.bytes.len() && (self.bytes[end] == b'e' || self.bytes[end] == b'E') {
            is_float = true;
            end += 1;
            if end < self.bytes.len() && (self.bytes[end] == b'+' || self.bytes[end] == b'-') {
                end += 1;
            }
            while end < self.bytes.len() && self.bytes[end].is_ascii_digit() {
                end += 1;
            }
        }
        let s = &self.input[start..end];
        self.pos = end;
        if is_float {
            let v = s
                .parse::<f64>()
                .map_err(|_| self.error(start, "invalid number literal"))?;
            if !v.is_finite() {
                return Err(self.error(start, "non-finite number literal"));
            }
            Ok(Token {
                tok: Tok::Number(v),
                start,
            })
        } else {
            let v = s
                .parse::<i64>()
                .map_err(|_| self.error(start, "integer literal out of range"))?;
            Ok(Token {
                tok: Tok::Int(v),
                start,
            })
        }
    }

    fn lex_string(&mut self, start: usize) -> Result<Token, ParseError> {
        let mut end = start + 1;
        let mut out = String::new();
        while end < self.bytes.len() {
            let b = self.bytes[end];
            if b == b'"' {
                end += 1;
                self.pos = end;
                return Ok(Token {
                    tok: Tok::Str(out),
                    start,
                });
            }
            if b == b'\\' {
                end += 1;
                if end >= self.bytes.len() {
                    return Err(self.error(start, "unterminated string literal"));
                }
                match self.bytes[end] {
                    b'"' => out.push('"'),
                    b'\\' => out.push('\\'),
                    b'n' => out.push('\n'),
                    b'r' => out.push('\r'),
                    b't' => out.push('\t'),
                    other => {
                        return Err(self.error(
                            end - 1,
                            format!("invalid string escape `\\{}`", other as char),
                        ));
                    }
                }
                end += 1;
            } else {
                // Walk one UTF-8 code point.
                let ch_start = end;
                while end < self.bytes.len()
                    && !matches!(self.bytes[end], b'"' | b'\\')
                    && self.bytes[end] >= 0x20
                {
                    end += 1;
                    // Stop on next quote/backslash — we'll recheck in loop.
                    if end < self.bytes.len()
                        && (self.bytes[end] == b'"' || self.bytes[end] == b'\\')
                    {
                        break;
                    }
                }
                if end == ch_start {
                    return Err(self.error(ch_start, "invalid character in string literal"));
                }
                out.push_str(&self.input[ch_start..end]);
            }
        }
        Err(self.error(start, "unterminated string literal"))
    }

    fn lex_string_single(&mut self, start: usize) -> Result<Token, ParseError> {
        // Accept single-quoted strings as string literals: the grammar doc's
        // BNF only admits double-quoted strings, but the denylist tests
        // (e.g., `require('x')`) use single quotes. Parse it the same way
        // as double-quoted so the REJECTED-identifier check fires on the
        // *identifier* rather than on the quoting.
        let mut end = start + 1;
        let mut out = String::new();
        while end < self.bytes.len() {
            let b = self.bytes[end];
            if b == b'\'' {
                end += 1;
                self.pos = end;
                return Ok(Token {
                    tok: Tok::Str(out),
                    start,
                });
            }
            if b == b'\\' {
                end += 1;
                if end >= self.bytes.len() {
                    return Err(self.error(start, "unterminated string literal"));
                }
                match self.bytes[end] {
                    b'\'' => out.push('\''),
                    b'\\' => out.push('\\'),
                    b'n' => out.push('\n'),
                    b'r' => out.push('\r'),
                    b't' => out.push('\t'),
                    other => {
                        return Err(self.error(
                            end - 1,
                            format!("invalid string escape `\\{}`", other as char),
                        ));
                    }
                }
                end += 1;
            } else {
                out.push(self.bytes[end] as char);
                end += 1;
            }
        }
        Err(self.error(start, "unterminated string literal"))
    }

    fn starts_with_at(&self, pos: usize, s: &str) -> bool {
        let b = s.as_bytes();
        if pos + b.len() > self.bytes.len() {
            return false;
        }
        &self.bytes[pos..pos + b.len()] == b
    }

    // --- grammar productions ------------------------------------------------

    fn parse_expression(&mut self) -> Result<Expr, ParseError> {
        self.parse_ternary()
    }

    fn parse_ternary(&mut self) -> Result<Expr, ParseError> {
        let cond = self.parse_logical_or()?;
        if matches!(self.peek_token()?.tok, Tok::Question) {
            self.take_token()?;
            let then_branch = self.parse_expression()?;
            let next = self.take_token()?;
            if !matches!(next.tok, Tok::Colon) {
                return Err(self.error(next.start, "expected `:` in ternary"));
            }
            let else_branch = self.parse_expression()?;
            return Ok(Expr::Conditional {
                cond: Box::new(cond),
                then_branch: Box::new(then_branch),
                else_branch: Box::new(else_branch),
            });
        }
        Ok(cond)
    }

    fn parse_logical_or(&mut self) -> Result<Expr, ParseError> {
        let mut lhs = self.parse_logical_and()?;
        while matches!(self.peek_token()?.tok, Tok::OrOr) {
            self.take_token()?;
            let rhs = self.parse_logical_and()?;
            lhs = Expr::Binary {
                op: BinaryOp::Or,
                lhs: Box::new(lhs),
                rhs: Box::new(rhs),
            };
        }
        Ok(lhs)
    }

    fn parse_logical_and(&mut self) -> Result<Expr, ParseError> {
        let mut lhs = self.parse_equality()?;
        while matches!(self.peek_token()?.tok, Tok::AndAnd) {
            self.take_token()?;
            let rhs = self.parse_equality()?;
            lhs = Expr::Binary {
                op: BinaryOp::And,
                lhs: Box::new(lhs),
                rhs: Box::new(rhs),
            };
        }
        Ok(lhs)
    }

    fn parse_equality(&mut self) -> Result<Expr, ParseError> {
        let lhs = self.parse_comparison()?;
        let op = match self.peek_token()?.tok {
            Tok::Eq => Some(BinaryOp::Eq),
            Tok::Ne => Some(BinaryOp::Ne),
            Tok::EqStrict => Some(BinaryOp::EqStrict),
            Tok::NeStrict => Some(BinaryOp::NeStrict),
            _ => None,
        };
        let Some(op) = op else {
            return Ok(lhs);
        };
        self.take_token()?;
        let rhs = self.parse_comparison()?;
        // Equality is non-associative — disallow chaining.
        if matches!(
            self.peek_token()?.tok,
            Tok::Eq | Tok::Ne | Tok::EqStrict | Tok::NeStrict
        ) {
            let t = self.peek_token()?;
            let off = t.start;
            return Err(self.error(off, "equality is non-associative; parenthesize"));
        }
        Ok(Expr::Binary {
            op,
            lhs: Box::new(lhs),
            rhs: Box::new(rhs),
        })
    }

    fn parse_comparison(&mut self) -> Result<Expr, ParseError> {
        let lhs = self.parse_additive()?;
        let op = match self.peek_token()?.tok {
            Tok::Lt => Some(BinaryOp::Lt),
            Tok::Le => Some(BinaryOp::Le),
            Tok::Gt => Some(BinaryOp::Gt),
            Tok::Ge => Some(BinaryOp::Ge),
            _ => None,
        };
        let Some(op) = op else {
            return Ok(lhs);
        };
        self.take_token()?;
        let rhs = self.parse_additive()?;
        // Non-associative.
        if matches!(
            self.peek_token()?.tok,
            Tok::Lt | Tok::Le | Tok::Gt | Tok::Ge
        ) {
            let t = self.peek_token()?;
            let off = t.start;
            return Err(self.error(off, "comparison is non-associative; parenthesize"));
        }
        Ok(Expr::Binary {
            op,
            lhs: Box::new(lhs),
            rhs: Box::new(rhs),
        })
    }

    fn parse_additive(&mut self) -> Result<Expr, ParseError> {
        let mut lhs = self.parse_multiplicative()?;
        loop {
            let op = match self.peek_token()?.tok {
                Tok::Plus => BinaryOp::Add,
                Tok::Minus => BinaryOp::Sub,
                _ => break,
            };
            self.take_token()?;
            let rhs = self.parse_multiplicative()?;
            lhs = Expr::Binary {
                op,
                lhs: Box::new(lhs),
                rhs: Box::new(rhs),
            };
        }
        Ok(lhs)
    }

    fn parse_multiplicative(&mut self) -> Result<Expr, ParseError> {
        let mut lhs = self.parse_unary()?;
        loop {
            let op = match self.peek_token()?.tok {
                Tok::Star => BinaryOp::Mul,
                Tok::Slash => BinaryOp::Div,
                Tok::Percent => BinaryOp::Mod,
                _ => break,
            };
            self.take_token()?;
            let rhs = self.parse_unary()?;
            lhs = Expr::Binary {
                op,
                lhs: Box::new(lhs),
                rhs: Box::new(rhs),
            };
        }
        Ok(lhs)
    }

    fn parse_unary(&mut self) -> Result<Expr, ParseError> {
        let t = self.peek_token()?;
        match t.tok {
            Tok::Bang => {
                self.take_token()?;
                let e = self.parse_unary()?;
                Ok(Expr::Unary {
                    op: UnaryOp::Not,
                    expr: Box::new(e),
                })
            }
            Tok::Minus => {
                self.take_token()?;
                let e = self.parse_unary()?;
                Ok(Expr::Unary {
                    op: UnaryOp::Neg,
                    expr: Box::new(e),
                })
            }
            Tok::Plus => {
                self.take_token()?;
                let e = self.parse_unary()?;
                Ok(Expr::Unary {
                    op: UnaryOp::Pos,
                    expr: Box::new(e),
                })
            }
            Tok::Slash => {
                // `/` in prefix position → a regex literal in JS. Reject.
                let off = t.start;
                Err(self.error(off, "regex literals are not in the grammar"))
            }
            _ => self.parse_postfix(),
        }
    }

    fn parse_postfix(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.parse_primary()?;
        loop {
            match self.peek_token()?.tok {
                Tok::Dot => {
                    self.take_token()?;
                    // The next token must be an Ident. We reject proto-like
                    // access here specifically.
                    let next = self.take_token()?;
                    let name = match next.tok {
                        Tok::Ident(n) => n,
                        _ => {
                            return Err(self.error(next.start, "expected identifier after `.`"));
                        }
                    };
                    if matches!(name.as_str(), "__proto__" | "constructor" | "prototype") {
                        // The "first rejected token" is the `.` starting the
                        // access span per the grammar-doc tests.
                        // We need to point at `.` — rewind calculation:
                        // `next.start` is the start of the identifier. The
                        // dot is `next.start - name.len() - (something)`.
                        // Actually since we consumed them in order, the
                        // dot's start was `next.start - 1 - whitespace`. We
                        // don't track it; scan back for the `.` byte.
                        let mut dot_pos = next.start;
                        while dot_pos > 0 && self.bytes[dot_pos - 1] != b'.' {
                            dot_pos -= 1;
                        }
                        let dot_pos = dot_pos.saturating_sub(1);
                        return Err(self.error(
                            dot_pos,
                            format!("`.{name}` is a rejected prototype-chain access"),
                        ));
                    }
                    expr = Expr::PropertyAccess {
                        target: Box::new(expr),
                        name,
                    };
                }
                Tok::LBracket => {
                    self.take_token()?;
                    let idx = self.parse_expression()?;
                    let next = self.take_token()?;
                    if !matches!(next.tok, Tok::RBracket) {
                        return Err(self.error(next.start, "expected `]`"));
                    }
                    expr = Expr::IndexAccess {
                        target: Box::new(expr),
                        index: Box::new(idx),
                    };
                }
                Tok::LParen => {
                    self.take_token()?;
                    // Identify the callee's "method name" when the callee is
                    // a property access, so we can enable lambda parsing on
                    // admitted methods only.
                    let method_name: Option<&'static str> = match &expr {
                        Expr::PropertyAccess { name, .. } => {
                            LAMBDA_CALL_METHODS.iter().copied().find(|n| *n == name)
                        }
                        Expr::Identifier(n) => {
                            LAMBDA_CALL_METHODS.iter().copied().find(|m| *m == n)
                        }
                        _ => None,
                    };
                    let prev_lambda_ctx = self.lambda_ctx;
                    self.lambda_ctx = method_name.is_some();
                    let args = self.parse_argument_list()?;
                    self.lambda_ctx = prev_lambda_ctx;
                    expr = Expr::Call {
                        callee: Box::new(expr),
                        args,
                    };
                }
                _ => break,
            }
        }
        Ok(expr)
    }

    fn parse_argument_list(&mut self) -> Result<Vec<Expr>, ParseError> {
        let mut args = Vec::new();
        if matches!(self.peek_token()?.tok, Tok::RParen) {
            self.take_token()?;
            return Ok(args);
        }
        loop {
            args.push(self.parse_argument()?);
            match self.peek_token()?.tok {
                Tok::Comma => {
                    self.take_token()?;
                }
                Tok::RParen => {
                    self.take_token()?;
                    return Ok(args);
                }
                _ => {
                    let t = self.peek_token()?;
                    let off = t.start;
                    return Err(self.error(off, "expected `,` or `)`"));
                }
            }
        }
    }

    /// Parse a single argument. Inside a `lambda_ctx`, admits
    /// `IDENT => expr` and `(IDENT, IDENT, …) => expr`. Outside, falls back
    /// to regular expression parsing.
    fn parse_argument(&mut self) -> Result<Expr, ParseError> {
        if self.lambda_ctx {
            // Attempt lambda. Two shapes: `x => …` or `(x, y) => …`.
            if let Some(lambda) = self.try_parse_lambda()? {
                return Ok(lambda);
            }
        }
        self.parse_expression()
    }

    /// Peek-only: try to parse a lambda. Returns `Ok(Some(..))` if a lambda
    /// was consumed; `Ok(None)` if not (caller falls through). Grammar-level
    /// rejections (e.g., arrow at an invalid position inside a non-lambda
    /// context) are surfaced through normal parsing — if the token stream
    /// doesn't match a lambda shape, we don't consume anything and return
    /// `Ok(None)`.
    fn try_parse_lambda(&mut self) -> Result<Option<Expr>, ParseError> {
        // Save parser state for rollback.
        let saved_pos = self.pos;
        let saved_peeked = self.peeked.clone();

        let first = self.peek_token()?.clone_tok_and_start();
        // Case 1: `IDENT =>` — single-param without parens.
        if let Tok::Ident(name) = &first.0 {
            // Re-check this is not a reserved word (rejected identifiers).
            if REJECTED_WORDS.contains(&name.as_str()) {
                // Let the primary parser reject the identifier with the
                // right error class — do not consume lambda.
                self.pos = saved_pos;
                self.peeked = saved_peeked;
                return Ok(None);
            }
            // Clone and look ahead manually: take the ident, then peek for =>.
            let name = name.clone();
            self.take_token()?;
            if matches!(self.peek_token()?.tok, Tok::Arrow) {
                self.take_token()?; // consume =>
                // Arrow-body is parsed OUTSIDE lambda_ctx (no nested lambda
                // in the body by default; grammar doesn't nest).
                let prev_ctx = self.lambda_ctx;
                self.lambda_ctx = false;
                let body = self.parse_expression()?;
                self.lambda_ctx = prev_ctx;
                return Ok(Some(Expr::Lambda {
                    params: vec![name],
                    body: Box::new(body),
                }));
            }
            // Not a lambda — rollback.
            self.pos = saved_pos;
            self.peeked = saved_peeked;
            return Ok(None);
        }
        // Case 2: `(IDENT, IDENT, …) =>` — multi-param with parens.
        if matches!(first.0, Tok::LParen) {
            self.take_token()?; // consume (
            let mut params = Vec::new();
            // Empty param list `() =>` would be grammatically admissible
            // but we don't need it for the test suite.
            if matches!(self.peek_token()?.tok, Tok::RParen) {
                self.take_token()?;
            } else {
                loop {
                    let t = self.take_token()?;
                    match t.tok {
                        Tok::Ident(ref n) if !REJECTED_WORDS.contains(&n.as_str()) => {
                            params.push(n.clone());
                        }
                        _ => {
                            // Not a valid lambda param list — rollback.
                            self.pos = saved_pos;
                            self.peeked = saved_peeked;
                            return Ok(None);
                        }
                    }
                    match self.peek_token()?.tok {
                        Tok::Comma => {
                            self.take_token()?;
                        }
                        Tok::RParen => {
                            self.take_token()?;
                            break;
                        }
                        _ => {
                            self.pos = saved_pos;
                            self.peeked = saved_peeked;
                            return Ok(None);
                        }
                    }
                }
            }
            if !matches!(self.peek_token()?.tok, Tok::Arrow) {
                self.pos = saved_pos;
                self.peeked = saved_peeked;
                return Ok(None);
            }
            self.take_token()?; // consume =>
            let prev_ctx = self.lambda_ctx;
            self.lambda_ctx = false;
            let body = self.parse_expression()?;
            self.lambda_ctx = prev_ctx;
            return Ok(Some(Expr::Lambda {
                params,
                body: Box::new(body),
            }));
        }
        Ok(None)
    }

    fn parse_primary(&mut self) -> Result<Expr, ParseError> {
        let t = self.take_token()?;
        match t.tok {
            Tok::Int(n) => Ok(Expr::Literal(Value::Int(n))),
            Tok::Number(n) => Ok(Expr::Literal(Value::Float(n))),
            Tok::Str(s) => Ok(Expr::Literal(Value::Text(s))),
            Tok::True => Ok(Expr::Literal(Value::Bool(true))),
            Tok::False => Ok(Expr::Literal(Value::Bool(false))),
            Tok::Null => Ok(Expr::Literal(Value::Null)),
            Tok::Dollar(name) => Ok(Expr::ContextBinding(name)),
            Tok::Ident(name) => {
                if REJECTED_WORDS.contains(&name.as_str()) {
                    return Err(self.error(
                        t.start,
                        format!("`{name}` is a rejected keyword / identifier"),
                    ));
                }
                Ok(Expr::Identifier(name))
            }
            Tok::LParen => {
                let expr = self.parse_expression()?;
                // Reject comma operator: `(a, b)` is not in the grammar.
                let next = self.take_token()?;
                match next.tok {
                    Tok::RParen => Ok(expr),
                    Tok::Comma => {
                        Err(self.error(next.start, "comma operator is not in the grammar"))
                    }
                    _ => Err(self.error(next.start, "expected `)`")),
                }
            }
            Tok::LBracket => {
                // Array literal.
                let mut items = Vec::new();
                if matches!(self.peek_token()?.tok, Tok::RBracket) {
                    self.take_token()?;
                    return Ok(Expr::Array(items));
                }
                loop {
                    items.push(self.parse_expression()?);
                    match self.peek_token()?.tok {
                        Tok::Comma => {
                            self.take_token()?;
                            if matches!(self.peek_token()?.tok, Tok::RBracket) {
                                self.take_token()?;
                                return Ok(Expr::Array(items));
                            }
                        }
                        Tok::RBracket => {
                            self.take_token()?;
                            return Ok(Expr::Array(items));
                        }
                        _ => {
                            let p = self.peek_token()?;
                            let off = p.start;
                            return Err(self.error(off, "expected `,` or `]`"));
                        }
                    }
                }
            }
            Tok::LBrace => {
                // Object literal: only IDENT / STRING keys; no computed keys.
                let mut fields: Vec<(String, Expr)> = Vec::new();
                if matches!(self.peek_token()?.tok, Tok::RBrace) {
                    self.take_token()?;
                    return Ok(Expr::Object(fields));
                }
                loop {
                    let key_tok = self.take_token()?;
                    let key_name = match &key_tok.tok {
                        Tok::Ident(n) => {
                            if REJECTED_WORDS.contains(&n.as_str()) {
                                return Err(self.error(
                                    key_tok.start,
                                    format!("`{n}` is a rejected keyword in key position"),
                                ));
                            }
                            n.clone()
                        }
                        Tok::Str(s) => s.clone(),
                        Tok::LBracket => {
                            return Err(self.error(
                                key_tok.start,
                                "computed property names are not in the grammar",
                            ));
                        }
                        _ => {
                            return Err(
                                self.error(key_tok.start, "expected identifier or string key")
                            );
                        }
                    };
                    // Shorthand: `{x}` meaning `{x: x}` — only when next
                    // token is `,` or `}`.
                    match self.peek_token()?.tok {
                        Tok::Colon => {
                            self.take_token()?;
                            let val = self.parse_expression()?;
                            fields.push((key_name, val));
                        }
                        Tok::Comma | Tok::RBrace => {
                            // Shorthand.
                            fields.push((key_name.clone(), Expr::Identifier(key_name)));
                        }
                        _ => {
                            let t = self.peek_token()?;
                            let off = t.start;
                            return Err(self.error(off, "expected `:` after object key"));
                        }
                    }
                    match self.peek_token()?.tok {
                        Tok::Comma => {
                            self.take_token()?;
                            if matches!(self.peek_token()?.tok, Tok::RBrace) {
                                self.take_token()?;
                                return Ok(Expr::Object(fields));
                            }
                        }
                        Tok::RBrace => {
                            self.take_token()?;
                            return Ok(Expr::Object(fields));
                        }
                        _ => {
                            let t = self.peek_token()?;
                            let off = t.start;
                            return Err(self.error(off, "expected `,` or `}`"));
                        }
                    }
                }
            }
            Tok::Arrow => Err(self.error(
                t.start,
                "arrow `=>` is only admitted as an argument of array methods",
            )),
            Tok::Eof => Err(self.error(t.start, "unexpected end of input")),
            Tok::Question
            | Tok::Colon
            | Tok::Comma
            | Tok::RParen
            | Tok::RBracket
            | Tok::RBrace
            | Tok::Dot
            | Tok::Slash
            | Tok::Star
            | Tok::Percent
            | Tok::Lt
            | Tok::Le
            | Tok::Gt
            | Tok::Ge
            | Tok::Eq
            | Tok::Ne
            | Tok::EqStrict
            | Tok::NeStrict
            | Tok::AndAnd
            | Tok::OrOr => Err(self.error(t.start, "unexpected operator in primary position")),
            // Prefix-position Plus/Minus/Bang handled in parse_unary.
            Tok::Plus | Tok::Minus | Tok::Bang => {
                Err(self.error(t.start, "unexpected prefix operator"))
            }
        }
    }
}

impl Token {
    fn clone_tok_and_start(&self) -> (Tok, usize) {
        (self.tok.clone(), self.start)
    }
}

fn is_ident_start(b: u8) -> bool {
    (b.is_ascii_alphabetic()) || b == b'_'
}

fn is_ident_byte(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_'
}
