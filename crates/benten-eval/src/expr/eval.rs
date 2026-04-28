#![allow(
    clippy::cast_precision_loss,
    reason = "TRANSFORM's Number type is JS-like f64; i64→f64 is intentional and documented in docs/TRANSFORM-GRAMMAR.md"
)]
#![allow(
    clippy::float_cmp,
    reason = "exact equality is required for deterministic evaluation"
)]

//! Pure evaluator for parsed TRANSFORM expressions.
//!
//! Walks an [`Expr`] AST against a binding environment and produces a
//! [`Value`]. No engine access, no clock, no RNG, no I/O — the function is
//! a pure map from `(Expr, bindings) -> Value`. Determinism is the load-
//! bearing property.
//!
//! The evaluator is recursion-free at the AST level but uses the Rust stack
//! for recursive descent. Phase 1 expressions are bounded by the grammar
//! (no loops, no closures that capture outer state, array-method built-ins
//! consume finite arrays), so bounded recursion depth is safe.

use super::builtins;
use super::{BinaryOp, Expr, UnaryOp};
use benten_core::Value;
use std::collections::BTreeMap;

/// Evaluation error — kept intentionally small; the public surface surfaces
/// `E_TRANSFORM_RUNTIME` via the caller's conversion.
#[derive(Debug, Clone)]
pub struct EvalError {
    /// Human-readable diagnostic message for the runtime failure.
    pub message: String,
}

impl EvalError {
    pub(crate) fn new(msg: impl Into<String>) -> Self {
        Self {
            message: msg.into(),
        }
    }
}

impl std::fmt::Display for EvalError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for EvalError {}

/// Binding environment for TRANSFORM evaluation.
///
/// The environment layers context bindings (`$input`, `$result`, `$item`,
/// `$index`, `$results`, `$error`) and plain identifier bindings (lambda
/// parameters). Inner frames shadow outer.
#[derive(Debug, Clone, Default)]
pub struct Env {
    frames: Vec<BTreeMap<String, Value>>,
}

impl Env {
    /// Construct an empty environment with one bottom frame.
    #[must_use]
    pub fn new() -> Self {
        Self {
            frames: vec![BTreeMap::new()],
        }
    }

    /// Construct an environment seeded with `$input = input` on the
    /// bottom frame.
    #[must_use]
    pub fn with_input(input: Value) -> Self {
        let mut e = Self::new();
        e.set("$input", input);
        e
    }

    /// Bind `k = v` on the topmost frame.
    pub fn set(&mut self, k: impl Into<String>, v: Value) {
        if let Some(f) = self.frames.last_mut() {
            f.insert(k.into(), v);
        }
    }

    /// Look up `k` walking from the topmost frame down to the bottom;
    /// returns the first match (inner shadows outer).
    #[must_use]
    pub fn get(&self, k: &str) -> Option<&Value> {
        for f in self.frames.iter().rev() {
            if let Some(v) = f.get(k) {
                return Some(v);
            }
        }
        None
    }

    /// Push a fresh frame populated with `bindings`. Used when entering
    /// a lambda body so the lambda's parameter set shadows the outer
    /// scope.
    pub fn push(&mut self, bindings: BTreeMap<String, Value>) {
        self.frames.push(bindings);
    }

    /// Pop the topmost frame; the bottom frame is preserved (no-op
    /// when only the bottom frame remains).
    pub fn pop(&mut self) {
        if self.frames.len() > 1 {
            self.frames.pop();
        }
    }
}

/// Evaluate a parsed expression against the supplied environment.
///
/// # Errors
///
/// Returns [`EvalError`] when built-in calls fail type checks, when a
/// non-finite float arises, or when the expression references an unknown
/// identifier.
pub fn eval(expr: &Expr, env: &mut Env) -> Result<Value, EvalError> {
    match expr {
        Expr::Literal(v) => Ok(v.clone()),
        Expr::Identifier(name) => env
            .get(name)
            .cloned()
            .ok_or_else(|| EvalError::new(format!("unknown identifier `{name}`"))),
        Expr::ContextBinding(name) => env
            .get(name)
            .cloned()
            .ok_or_else(|| EvalError::new(format!("context binding `{name}` not set"))),
        Expr::Binary { op, lhs, rhs } => eval_binary(*op, lhs, rhs, env),
        Expr::Unary { op, expr } => {
            let v = eval(expr, env)?;
            eval_unary(*op, v)
        }
        Expr::Conditional {
            cond,
            then_branch,
            else_branch,
        } => {
            let c = eval(cond, env)?;
            if truthy(&c) {
                eval(then_branch, env)
            } else {
                eval(else_branch, env)
            }
        }
        Expr::PropertyAccess { target, name } => {
            let v = eval(target, env)?;
            property_access(&v, name)
        }
        Expr::IndexAccess { target, index } => {
            let t = eval(target, env)?;
            let i = eval(index, env)?;
            index_access(&t, &i)
        }
        Expr::Array(items) => {
            let mut out = Vec::with_capacity(items.len());
            for it in items {
                out.push(eval(it, env)?);
            }
            Ok(Value::List(out))
        }
        Expr::Object(fields) => {
            let mut out = BTreeMap::new();
            for (k, v) in fields {
                out.insert(k.clone(), eval(v, env)?);
            }
            Ok(Value::Map(out))
        }
        Expr::Call { callee, args } => eval_call(callee, args, env),
        Expr::Lambda { .. } => Err(EvalError::new(
            "lambda expression may only appear as an argument of array methods",
        )),
    }
}

fn eval_binary(op: BinaryOp, lhs: &Expr, rhs: &Expr, env: &mut Env) -> Result<Value, EvalError> {
    // Short-circuit logical ops.
    match op {
        BinaryOp::And => {
            let l = eval(lhs, env)?;
            if !truthy(&l) {
                return Ok(l);
            }
            return eval(rhs, env);
        }
        BinaryOp::Or => {
            let l = eval(lhs, env)?;
            if truthy(&l) {
                return Ok(l);
            }
            return eval(rhs, env);
        }
        _ => {}
    }
    let l = eval(lhs, env)?;
    let r = eval(rhs, env)?;
    match op {
        BinaryOp::Add => add(&l, &r),
        BinaryOp::Sub => numeric_op(&l, &r, |a, b| a - b, |a, b| a - b),
        BinaryOp::Mul => numeric_op(&l, &r, |a, b| a * b, |a, b| a * b),
        BinaryOp::Div => divide(&l, &r),
        BinaryOp::Mod => modulo(&l, &r),
        BinaryOp::Lt => Ok(Value::Bool(compare(&l, &r)? == std::cmp::Ordering::Less)),
        BinaryOp::Le => Ok(Value::Bool(compare(&l, &r)? != std::cmp::Ordering::Greater)),
        BinaryOp::Gt => Ok(Value::Bool(compare(&l, &r)? == std::cmp::Ordering::Greater)),
        BinaryOp::Ge => Ok(Value::Bool(compare(&l, &r)? != std::cmp::Ordering::Less)),
        BinaryOp::Eq | BinaryOp::EqStrict => Ok(Value::Bool(values_equal(&l, &r))),
        BinaryOp::Ne | BinaryOp::NeStrict => Ok(Value::Bool(!values_equal(&l, &r))),
        BinaryOp::And | BinaryOp::Or => unreachable!("short-circuited above"),
    }
}

fn eval_unary(op: UnaryOp, v: Value) -> Result<Value, EvalError> {
    match op {
        UnaryOp::Not => Ok(Value::Bool(!truthy(&v))),
        UnaryOp::Neg => match v {
            Value::Int(i) => Ok(Value::Int(-i)),
            Value::Float(f) => Ok(Value::Float(-f)),
            _ => Err(EvalError::new("unary `-` on non-numeric value")),
        },
        UnaryOp::Pos => match v {
            Value::Int(_) | Value::Float(_) => Ok(v),
            _ => Err(EvalError::new("unary `+` on non-numeric value")),
        },
    }
}

fn eval_call(callee: &Expr, args: &[Expr], env: &mut Env) -> Result<Value, EvalError> {
    // Namespaced built-in: `Math.min(a, b)` etc. Check first so we don't
    // try to resolve `Math` as an identifier binding.
    if let Some(v) = try_namespaced_call(callee, args, env)? {
        return Ok(v);
    }
    // Method-call shape: `obj.method(args)`. Evaluate the receiver and
    // dispatch based on method name.
    if let Expr::PropertyAccess { target, name } = callee {
        let receiver = eval(target, env)?;
        return dispatch_method(&receiver, name, args, env);
    }
    // Identifier call (built-in): `min(a, b)`, `lower(s)`, etc.
    if let Expr::Identifier(name) = callee {
        let mut arg_vals = Vec::with_capacity(args.len());
        for a in args {
            arg_vals.push(eval(a, env)?);
        }
        return builtins::dispatch_builtin(name, &arg_vals);
    }
    Err(EvalError::new("unsupported call form"))
}

fn dispatch_method(
    receiver: &Value,
    method: &str,
    args: &[Expr],
    env: &mut Env,
) -> Result<Value, EvalError> {
    // Namespaced Math / String / Date / Array pseudo-method calls: the
    // `receiver` is an Identifier reference that the evaluator has already
    // resolved. If the target is Null and the Expr::PropertyAccess target
    // was an Identifier like "Math", look up a namespaced builtin.
    // We handle these separately to keep the JS-familiar surface.
    if let Value::Null = receiver {
        // Not a receiver; rely on top-level identifier dispatch. Fall through.
    }

    // Method-name hash for common string/array/object methods.
    match method {
        // --- String methods ---------------------------------------------
        "toLowerCase" => {
            if !args.is_empty() {
                return Err(EvalError::new("toLowerCase takes no arguments"));
            }
            match receiver {
                Value::Text(s) => Ok(Value::Text(s.to_lowercase())),
                _ => Err(EvalError::new("toLowerCase on non-string")),
            }
        }
        "toUpperCase" => {
            if !args.is_empty() {
                return Err(EvalError::new("toUpperCase takes no arguments"));
            }
            match receiver {
                Value::Text(s) => Ok(Value::Text(s.to_uppercase())),
                _ => Err(EvalError::new("toUpperCase on non-string")),
            }
        }
        "trim" => {
            if !args.is_empty() {
                return Err(EvalError::new("trim takes no arguments"));
            }
            match receiver {
                Value::Text(s) => Ok(Value::Text(s.trim().to_string())),
                _ => Err(EvalError::new("trim on non-string")),
            }
        }
        "startsWith" => {
            let needle = eval_single_arg(args, env)?;
            match (receiver, &needle) {
                (Value::Text(s), Value::Text(n)) => Ok(Value::Bool(s.starts_with(n.as_str()))),
                _ => Err(EvalError::new("startsWith type mismatch")),
            }
        }
        "endsWith" => {
            let needle = eval_single_arg(args, env)?;
            match (receiver, &needle) {
                (Value::Text(s), Value::Text(n)) => Ok(Value::Bool(s.ends_with(n.as_str()))),
                _ => Err(EvalError::new("endsWith type mismatch")),
            }
        }
        // --- Array methods ---------------------------------------------
        "map" => array_map(receiver, args, env),
        "filter" => array_filter(receiver, args, env),
        "reduce" => array_reduce(receiver, args, env),
        "find" => array_find(receiver, args, env),
        "findIndex" => array_find_index(receiver, args, env),
        "every" => array_every(receiver, args, env),
        "some" => array_some(receiver, args, env),
        "slice" => {
            let arg_vals = eval_args(args, env)?;
            builtins::array_slice(receiver, &arg_vals)
        }
        "concat" => {
            let arg_vals = eval_args(args, env)?;
            builtins::array_concat(receiver, &arg_vals)
        }
        // --- Math / namespaced built-in dispatch -----------------------
        // When called as `Math.min(a, b)`, `receiver` is whatever `Math`
        // evaluates to. We treat `Math`, `String`, `Date`, `Array`,
        // `Object`, `Number` as namespace identifiers that produce a
        // sentinel map; instead, we dispatch them via the parent
        // eval_call path by inspecting the callee shape.
        _ => {
            // Property-access-style namespace dispatch. If the receiver is
            // a `Value::Text` carrying a well-known namespace sentinel, we
            // wouldn't have produced it — so we instead look up the method
            // as a namespaced builtin using (namespace, method). This is
            // handled in the `eval_call_with_namespace` helper.
            Err(EvalError::new(format!(
                "unknown method `.{method}` on value `{receiver:?}`"
            )))
        }
    }
}

fn eval_args(args: &[Expr], env: &mut Env) -> Result<Vec<Value>, EvalError> {
    let mut out = Vec::with_capacity(args.len());
    for a in args {
        out.push(eval(a, env)?);
    }
    Ok(out)
}

fn eval_single_arg(args: &[Expr], env: &mut Env) -> Result<Value, EvalError> {
    if args.len() != 1 {
        return Err(EvalError::new("method expects exactly 1 argument"));
    }
    eval(&args[0], env)
}

// ---------------------------------------------------------------------------
// Array higher-order methods with lambda arguments.
// ---------------------------------------------------------------------------

fn array_map(receiver: &Value, args: &[Expr], env: &mut Env) -> Result<Value, EvalError> {
    let items = as_list(receiver, "map")?;
    let (params, body) = lambda_arg(args, 0, "map")?;
    let mut out = Vec::with_capacity(items.len());
    for item in items {
        apply_lambda_binding(env, params, std::slice::from_ref(item));
        let v = eval(body, env);
        env.pop();
        out.push(v?);
    }
    Ok(Value::List(out))
}

fn array_filter(receiver: &Value, args: &[Expr], env: &mut Env) -> Result<Value, EvalError> {
    let items = as_list(receiver, "filter")?;
    let (params, body) = lambda_arg(args, 0, "filter")?;
    let mut out = Vec::new();
    for item in items {
        apply_lambda_binding(env, params, std::slice::from_ref(item));
        let v = eval(body, env);
        env.pop();
        if truthy(&v?) {
            out.push(item.clone());
        }
    }
    Ok(Value::List(out))
}

fn array_reduce(receiver: &Value, args: &[Expr], env: &mut Env) -> Result<Value, EvalError> {
    let items = as_list(receiver, "reduce")?;
    if args.len() != 2 {
        return Err(EvalError::new("reduce takes a lambda and an initial value"));
    }
    let (params, body) = lambda_arg(args, 0, "reduce")?;
    let mut acc = eval(&args[1], env)?;
    for item in items {
        apply_lambda_binding(env, &params, &[acc.clone(), item.clone()]);
        let v = eval(body, env);
        env.pop();
        acc = v?;
    }
    Ok(acc)
}

fn array_find(receiver: &Value, args: &[Expr], env: &mut Env) -> Result<Value, EvalError> {
    let items = as_list(receiver, "find")?;
    let (params, body) = lambda_arg(args, 0, "find")?;
    for item in items {
        apply_lambda_binding(env, params, std::slice::from_ref(item));
        let v = eval(body, env);
        env.pop();
        if truthy(&v?) {
            return Ok(item.clone());
        }
    }
    Ok(Value::Null)
}

fn array_find_index(receiver: &Value, args: &[Expr], env: &mut Env) -> Result<Value, EvalError> {
    let items = as_list(receiver, "findIndex")?;
    let (params, body) = lambda_arg(args, 0, "findIndex")?;
    for (i, item) in items.iter().enumerate() {
        apply_lambda_binding(env, params, std::slice::from_ref(item));
        let v = eval(body, env);
        env.pop();
        if truthy(&v?) {
            let idx_i64 = i64::try_from(i)
                .map_err(|_| EvalError::new("findIndex: index exceeds i64 range"))?;
            return Ok(Value::Int(idx_i64));
        }
    }
    Ok(Value::Int(-1))
}

fn array_every(receiver: &Value, args: &[Expr], env: &mut Env) -> Result<Value, EvalError> {
    let items = as_list(receiver, "every")?;
    let (params, body) = lambda_arg(args, 0, "every")?;
    for item in items {
        apply_lambda_binding(env, params, std::slice::from_ref(item));
        let v = eval(body, env);
        env.pop();
        if !truthy(&v?) {
            return Ok(Value::Bool(false));
        }
    }
    Ok(Value::Bool(true))
}

fn array_some(receiver: &Value, args: &[Expr], env: &mut Env) -> Result<Value, EvalError> {
    let items = as_list(receiver, "some")?;
    let (params, body) = lambda_arg(args, 0, "some")?;
    for item in items {
        apply_lambda_binding(env, params, std::slice::from_ref(item));
        let v = eval(body, env);
        env.pop();
        if truthy(&v?) {
            return Ok(Value::Bool(true));
        }
    }
    Ok(Value::Bool(false))
}

fn as_list<'a>(v: &'a Value, method: &str) -> Result<&'a [Value], EvalError> {
    match v {
        Value::List(l) => Ok(l.as_slice()),
        _ => Err(EvalError::new(format!("{method} on non-array"))),
    }
}

fn lambda_arg<'a>(
    args: &'a [Expr],
    idx: usize,
    method: &str,
) -> Result<(&'a [String], &'a Expr), EvalError> {
    let Some(arg) = args.get(idx) else {
        return Err(EvalError::new(format!("{method} missing lambda argument")));
    };
    match arg {
        Expr::Lambda { params, body } => Ok((params.as_slice(), body.as_ref())),
        _ => Err(EvalError::new(format!(
            "{method} expects a lambda `x => …` argument"
        ))),
    }
}

fn apply_lambda_binding(env: &mut Env, params: &[String], values: &[Value]) {
    let mut frame = BTreeMap::new();
    for (i, p) in params.iter().enumerate() {
        let v = values.get(i).cloned().unwrap_or(Value::Null);
        frame.insert(p.clone(), v);
    }
    env.push(frame);
}

// ---------------------------------------------------------------------------
// Operator primitives.
// ---------------------------------------------------------------------------

fn truthy(v: &Value) -> bool {
    match v {
        Value::Null => false,
        Value::Bool(b) => *b,
        Value::Int(i) => *i != 0,
        Value::Float(f) => *f != 0.0 && !f.is_nan(),
        Value::Text(s) => !s.is_empty(),
        Value::Bytes(b) => !b.is_empty(),
        Value::List(l) => !l.is_empty(),
        Value::Map(m) => !m.is_empty(),
    }
}

fn add(l: &Value, r: &Value) -> Result<Value, EvalError> {
    match (l, r) {
        (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a.wrapping_add(*b))),
        (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a + b)),
        (Value::Int(a), Value::Float(b)) => Ok(Value::Float((*a as f64) + b)),
        (Value::Float(a), Value::Int(b)) => Ok(Value::Float(a + (*b as f64))),
        (Value::Text(a), Value::Text(b)) => Ok(Value::Text(format!("{a}{b}"))),
        _ => Err(EvalError::new("+ type mismatch")),
    }
}

fn numeric_op(
    l: &Value,
    r: &Value,
    op_i: fn(i64, i64) -> i64,
    op_f: fn(f64, f64) -> f64,
) -> Result<Value, EvalError> {
    match (l, r) {
        (Value::Int(a), Value::Int(b)) => Ok(Value::Int(op_i(*a, *b))),
        (Value::Float(a), Value::Float(b)) => Ok(Value::Float(op_f(*a, *b))),
        (Value::Int(a), Value::Float(b)) => Ok(Value::Float(op_f(*a as f64, *b))),
        (Value::Float(a), Value::Int(b)) => Ok(Value::Float(op_f(*a, *b as f64))),
        _ => Err(EvalError::new("numeric op type mismatch")),
    }
}

fn divide(l: &Value, r: &Value) -> Result<Value, EvalError> {
    match (l, r) {
        (Value::Int(a), Value::Int(b)) => {
            if *b == 0 {
                return Err(EvalError::new("division by zero"));
            }
            // Integer division if evenly divisible; else promote to float.
            if a % b == 0 {
                Ok(Value::Int(a / b))
            } else {
                Ok(Value::Float((*a as f64) / (*b as f64)))
            }
        }
        (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a / b)),
        (Value::Int(a), Value::Float(b)) => Ok(Value::Float((*a as f64) / b)),
        (Value::Float(a), Value::Int(b)) => Ok(Value::Float(a / (*b as f64))),
        _ => Err(EvalError::new("/ type mismatch")),
    }
}

fn modulo(l: &Value, r: &Value) -> Result<Value, EvalError> {
    match (l, r) {
        (Value::Int(a), Value::Int(b)) => {
            if *b == 0 {
                return Err(EvalError::new("modulo by zero"));
            }
            Ok(Value::Int(a % b))
        }
        (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a % b)),
        (Value::Int(a), Value::Float(b)) => Ok(Value::Float((*a as f64) % b)),
        (Value::Float(a), Value::Int(b)) => Ok(Value::Float(a % (*b as f64))),
        _ => Err(EvalError::new("% type mismatch")),
    }
}

fn compare(l: &Value, r: &Value) -> Result<std::cmp::Ordering, EvalError> {
    match (l, r) {
        (Value::Int(a), Value::Int(b)) => Ok(a.cmp(b)),
        (Value::Float(a), Value::Float(b)) => a
            .partial_cmp(b)
            .ok_or_else(|| EvalError::new("NaN in comparison")),
        (Value::Int(a), Value::Float(b)) => (*a as f64)
            .partial_cmp(b)
            .ok_or_else(|| EvalError::new("NaN in comparison")),
        (Value::Float(a), Value::Int(b)) => a
            .partial_cmp(&(*b as f64))
            .ok_or_else(|| EvalError::new("NaN in comparison")),
        (Value::Text(a), Value::Text(b)) => Ok(a.cmp(b)),
        _ => Err(EvalError::new("comparison type mismatch")),
    }
}

fn values_equal(l: &Value, r: &Value) -> bool {
    match (l, r) {
        (Value::Int(a), Value::Float(b)) => (*a as f64) == *b,
        (Value::Float(a), Value::Int(b)) => *a == (*b as f64),
        _ => l == r,
    }
}

fn property_access(v: &Value, name: &str) -> Result<Value, EvalError> {
    match v {
        Value::Map(m) => Ok(m.get(name).cloned().unwrap_or(Value::Null)),
        Value::List(l) if name == "length" => {
            let n = i64::try_from(l.len())
                .map_err(|_| EvalError::new("list length exceeds i64 range"))?;
            Ok(Value::Int(n))
        }
        Value::Text(s) if name == "length" => {
            let n = i64::try_from(s.chars().count())
                .map_err(|_| EvalError::new("string length exceeds i64 range"))?;
            Ok(Value::Int(n))
        }
        _ => Err(EvalError::new(format!(
            "property `.{name}` not accessible on value"
        ))),
    }
}

fn index_access(target: &Value, idx: &Value) -> Result<Value, EvalError> {
    match (target, idx) {
        (Value::List(l), Value::Int(i)) => {
            let len = l.len();
            let n = i64::try_from(len).map_err(|_| EvalError::new("list length overflow"))?;
            let abs = if *i < 0 { n + i } else { *i };
            if abs < 0 {
                return Ok(Value::Null);
            }
            let abs = usize::try_from(abs).map_err(|_| EvalError::new("index out of range"))?;
            Ok(l.get(abs).cloned().unwrap_or(Value::Null))
        }
        (Value::Map(m), Value::Text(k)) => Ok(m.get(k).cloned().unwrap_or(Value::Null)),
        (Value::Text(s), Value::Int(i)) => {
            let chars: Vec<char> = s.chars().collect();
            let n =
                i64::try_from(chars.len()).map_err(|_| EvalError::new("string length overflow"))?;
            let abs = if *i < 0 { n + i } else { *i };
            if abs < 0 {
                return Ok(Value::Null);
            }
            let abs = usize::try_from(abs).map_err(|_| EvalError::new("index out of range"))?;
            Ok(chars
                .get(abs)
                .map_or(Value::Null, |c| Value::Text(c.to_string())))
        }
        _ => Err(EvalError::new("index access type mismatch")),
    }
}

// ---------------------------------------------------------------------------
// Namespace dispatch (Math.*, String.*, etc.).
// ---------------------------------------------------------------------------

/// Resolve a property-access-call of the form `Namespace.method(args)` into
/// the corresponding namespaced built-in. Called by the evaluator before
/// falling back to method dispatch on an evaluated receiver, so that
/// `Math.min` doesn't require `Math` to be a real binding.
pub fn try_namespaced_call(
    callee: &Expr,
    args: &[Expr],
    env: &mut Env,
) -> Result<Option<Value>, EvalError> {
    if let Expr::PropertyAccess { target, name } = callee
        && let Expr::Identifier(ns) = target.as_ref()
        && matches!(
            ns.as_str(),
            "Math" | "String" | "Array" | "Object" | "Number"
        )
    {
        let arg_vals = eval_args(args, env)?;
        let full = format!("{ns}.{name}");
        return Ok(Some(builtins::dispatch_namespaced(&full, &arg_vals)?));
    }
    Ok(None)
}

/// Entry point used by `primitives::transform` — parse, then evaluate.
///
/// # Errors
///
/// Returns [`EvalError`] on any runtime failure; callers map this to
/// `E_TRANSFORM_RUNTIME` on the evaluator's error edge.
pub fn eval_with_namespaces(expr: &Expr, env: &mut Env) -> Result<Value, EvalError> {
    // Intercept calls before property-access evaluation to handle the
    // `Namespace.method(args)` sugar form.
    if let Expr::Call { callee, args } = expr
        && let Some(v) = try_namespaced_call(callee, args, env)?
    {
        return Ok(v);
    }
    eval(expr, env)
}
