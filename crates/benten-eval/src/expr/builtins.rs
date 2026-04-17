#![allow(
    clippy::cast_precision_loss,
    reason = "TRANSFORM's Number type is JS-like f64; i64→f64 is intentional and documented in docs/TRANSFORM-GRAMMAR.md"
)]
#![allow(
    clippy::float_cmp,
    reason = "exact equality is required for deterministic evaluation; NaN handling is explicit"
)]

//! TRANSFORM built-in call dispatchers (50+ functions per grammar doc).
//!
//! Dispatched in two forms:
//!
//! - **Bare builtin** — `min(a, b)`, `lower(s)`, `length(arr)`, etc. Looked
//!   up by the plain name via [`dispatch_builtin`].
//! - **Namespaced builtin** — `Math.min`, `String.lower`, `Array.length`,
//!   `Object.keys`, `Number.toNumber`. Looked up via
//!   [`dispatch_namespaced`]. The namespaced form is admitted for
//!   JavaScript-familiar aesthetics; the grammar doc's canonical form is
//!   the bare name.
//!
//! All built-ins are pure and deterministic. Functions that would introduce
//! nondeterminism (time, RNG, locale-sensitive case folding) are rejected
//! by the grammar or omitted.

use super::eval::EvalError;
use benten_core::Value;
use std::collections::BTreeMap;

/// Dispatch a bare built-in call like `min(a, b)`.
///
/// # Errors
///
/// Returns [`EvalError`] if the name isn't an admitted built-in or if the
/// argument types fail the built-in's contract.
pub fn dispatch_builtin(name: &str, args: &[Value]) -> Result<Value, EvalError> {
    match name {
        // --- Arithmetic ---
        "abs" => arith_abs(args),
        "ceil" => arith_ceil(args),
        "floor" => arith_floor(args),
        "round" => arith_round(args),
        "min" => arith_min(args),
        "max" => arith_max(args),
        "sum" => arith_sum(args),
        "product" => arith_product(args),
        "sqrt" => arith_sqrt(args),
        "pow" => arith_pow(args),
        "log" => arith_log(args, f64::ln),
        "log10" => arith_log(args, f64::log10),
        "log2" => arith_log(args, f64::log2),
        "exp" => arith_log(args, f64::exp),
        "sign" => arith_sign(args),
        "trunc" => arith_trunc(args),
        // --- String ---
        "length" => fn_length(args),
        "upper" => str_upper(args),
        "lower" => str_lower(args),
        "trim" => str_trim(args),
        "trimStart" => str_trim_start(args),
        "trimEnd" => str_trim_end(args),
        "startsWith" => str_starts_with(args),
        "endsWith" => str_ends_with(args),
        "contains" => str_contains(args),
        "substring" => str_substring(args),
        "replace" => str_replace(args),
        "split" => str_split(args),
        "join" => str_join(args),
        "padStart" => str_pad_start(args),
        "padEnd" => str_pad_end(args),
        "truncate" => str_truncate(args),
        // --- Array (non-lambda; lambda array ops dispatched via method) ---
        "first" => arr_first(args),
        "last" => arr_last(args),
        "at" => arr_at(args),
        "slice" => array_slice_fn(args),
        "concat" => array_concat_fn(args),
        "reverse" => arr_reverse(args),
        "sort" => arr_sort(args),
        "unique" => arr_unique(args),
        "flatten" => arr_flatten(args),
        "take" => arr_take(args),
        "skip" => arr_skip(args),
        // --- Object ---
        "keys" => obj_keys(args),
        "values" => obj_values(args),
        "entries" => obj_entries(args),
        "hasKey" => obj_has_key(args),
        "pick" => obj_pick(args),
        "omit" => obj_omit(args),
        "merge" => obj_merge(args),
        // --- Coercion ---
        "toNumber" => coerce_to_number(args),
        "toString" => coerce_to_string(args),
        "toArray" => coerce_to_array(args),
        "isNumber" => coerce_is_number(args),
        "isString" => coerce_is_string(args),
        "isArray" => coerce_is_array(args),
        "isObject" => coerce_is_object(args),
        "isNull" => coerce_is_null(args),
        "isEmpty" => coerce_is_empty(args),
        // --- Number formatting ---
        "formatNumber" => fmt_number(args),
        "formatPercent" => fmt_percent(args),
        "formatCurrency" => fmt_currency(args),
        _ => Err(EvalError::new(format!("unknown built-in `{name}`"))),
    }
}

/// Dispatch `Namespace.method(args)` sugar.
///
/// # Errors
///
/// Returns [`EvalError`] on unknown namespace methods or argument-type
/// mismatches.
pub fn dispatch_namespaced(full: &str, args: &[Value]) -> Result<Value, EvalError> {
    // Mirror the bare-builtin dispatch for common Math / String forms.
    match full {
        "Math.min" => arith_min(args),
        "Math.max" => arith_max(args),
        "Math.abs" => arith_abs(args),
        "Math.round" => arith_round(args),
        "Math.ceil" => arith_ceil(args),
        "Math.floor" => arith_floor(args),
        "Math.sqrt" => arith_sqrt(args),
        "Math.pow" => arith_pow(args),
        "Math.sign" => arith_sign(args),
        "Math.trunc" => arith_trunc(args),
        "String.lower" | "String.toLower" => str_lower(args),
        "String.upper" | "String.toUpper" => str_upper(args),
        "String.trim" => str_trim(args),
        "String.truncate" => str_truncate(args),
        "String.startsWith" => str_starts_with(args),
        "String.endsWith" => str_ends_with(args),
        "String.substring" => str_substring(args),
        "Array.from" => coerce_to_array(args),
        "Array.concat" => array_concat_fn(args),
        "Object.keys" => obj_keys(args),
        "Object.values" => obj_values(args),
        "Number.toNumber" => coerce_to_number(args),
        _ => Err(EvalError::new(format!(
            "unknown namespaced built-in `{full}`"
        ))),
    }
}

// Public re-exports used by the method dispatcher in `eval.rs`.

pub fn array_slice(receiver: &Value, args: &[Value]) -> Result<Value, EvalError> {
    let items = list_ref(receiver, "slice")?;
    let start = args.first().and_then(as_i64).unwrap_or(0);
    let end = args
        .get(1)
        .and_then(as_i64)
        .unwrap_or_else(|| i64::try_from(items.len()).unwrap_or(i64::MAX));
    let s = clamp_index(start, items.len());
    let e = clamp_index(end, items.len());
    if e <= s {
        Ok(Value::List(Vec::new()))
    } else {
        Ok(Value::List(items[s..e].to_vec()))
    }
}

pub fn array_concat(receiver: &Value, args: &[Value]) -> Result<Value, EvalError> {
    let mut out = match receiver {
        Value::List(l) => l.clone(),
        _ => return Err(EvalError::new("concat on non-array")),
    };
    for a in args {
        match a {
            Value::List(l) => out.extend(l.iter().cloned()),
            other => out.push(other.clone()),
        }
    }
    Ok(Value::List(out))
}

// ---------------------------------------------------------------------------
// Arithmetic
// ---------------------------------------------------------------------------

fn arith_abs(args: &[Value]) -> Result<Value, EvalError> {
    let v = single(args, "abs")?;
    match v {
        Value::Int(i) => Ok(Value::Int(i.abs())),
        Value::Float(f) => Ok(Value::Float(f.abs())),
        _ => Err(EvalError::new("abs on non-numeric")),
    }
}

fn arith_ceil(args: &[Value]) -> Result<Value, EvalError> {
    let f = to_f64(single(args, "ceil")?)?;
    Ok(Value::Int(f.ceil() as i64))
}

fn arith_floor(args: &[Value]) -> Result<Value, EvalError> {
    let f = to_f64(single(args, "floor")?)?;
    Ok(Value::Int(f.floor() as i64))
}

fn arith_round(args: &[Value]) -> Result<Value, EvalError> {
    let f = to_f64(single(args, "round")?)?;
    Ok(Value::Int(f.round() as i64))
}

fn arith_trunc(args: &[Value]) -> Result<Value, EvalError> {
    let f = to_f64(single(args, "trunc")?)?;
    Ok(Value::Int(f.trunc() as i64))
}

fn arith_min(args: &[Value]) -> Result<Value, EvalError> {
    if args.is_empty() {
        return Err(EvalError::new("min requires at least one argument"));
    }
    // Accept either `min(a, b, …)` or `min([…])`.
    let items: Vec<&Value> = if args.len() == 1 {
        if let Value::List(l) = &args[0] {
            l.iter().collect()
        } else {
            vec![&args[0]]
        }
    } else {
        args.iter().collect()
    };
    let mut best: Option<&Value> = None;
    for v in items {
        best = Some(match best {
            None => v,
            Some(b) => {
                if numeric_lt(v, b)? {
                    v
                } else {
                    b
                }
            }
        });
    }
    Ok(best.cloned().unwrap_or(Value::Null))
}

fn arith_max(args: &[Value]) -> Result<Value, EvalError> {
    if args.is_empty() {
        return Err(EvalError::new("max requires at least one argument"));
    }
    let items: Vec<&Value> = if args.len() == 1 {
        if let Value::List(l) = &args[0] {
            l.iter().collect()
        } else {
            vec![&args[0]]
        }
    } else {
        args.iter().collect()
    };
    let mut best: Option<&Value> = None;
    for v in items {
        best = Some(match best {
            None => v,
            Some(b) => {
                if numeric_lt(b, v)? {
                    v
                } else {
                    b
                }
            }
        });
    }
    Ok(best.cloned().unwrap_or(Value::Null))
}

fn arith_sum(args: &[Value]) -> Result<Value, EvalError> {
    let items = flatten_one(args);
    let mut total_i: i64 = 0;
    let mut total_f: f64 = 0.0;
    let mut any_float = false;
    for v in items {
        match v {
            Value::Int(i) => total_i = total_i.wrapping_add(i),
            Value::Float(f) => {
                any_float = true;
                total_f += f;
            }
            _ => return Err(EvalError::new("sum on non-numeric")),
        }
    }
    if any_float {
        Ok(Value::Float(total_f + total_i as f64))
    } else {
        Ok(Value::Int(total_i))
    }
}

fn arith_product(args: &[Value]) -> Result<Value, EvalError> {
    let items = flatten_one(args);
    let mut total_i: i64 = 1;
    let mut total_f: f64 = 1.0;
    let mut any_float = false;
    for v in items {
        match v {
            Value::Int(i) => total_i = total_i.wrapping_mul(i),
            Value::Float(f) => {
                any_float = true;
                total_f *= f;
            }
            _ => return Err(EvalError::new("product on non-numeric")),
        }
    }
    if any_float {
        Ok(Value::Float(total_f * total_i as f64))
    } else {
        Ok(Value::Int(total_i))
    }
}

fn arith_sqrt(args: &[Value]) -> Result<Value, EvalError> {
    let f = to_f64(single(args, "sqrt")?)?;
    Ok(Value::Float(f.sqrt()))
}

fn arith_pow(args: &[Value]) -> Result<Value, EvalError> {
    if args.len() != 2 {
        return Err(EvalError::new("pow takes exactly 2 arguments"));
    }
    let base = to_f64(args[0].clone())?;
    let exp = to_f64(args[1].clone())?;
    Ok(Value::Float(base.powf(exp)))
}

fn arith_log(args: &[Value], op: fn(f64) -> f64) -> Result<Value, EvalError> {
    let f = to_f64(single(args, "log")?)?;
    Ok(Value::Float(op(f)))
}

fn arith_sign(args: &[Value]) -> Result<Value, EvalError> {
    let v = single(args, "sign")?;
    match v {
        Value::Int(i) => Ok(Value::Int(i.signum())),
        Value::Float(f) => {
            if f.is_nan() {
                return Err(EvalError::new("sign(NaN)"));
            }
            Ok(Value::Int(if f > 0.0 {
                1
            } else if f < 0.0 {
                -1
            } else {
                0
            }))
        }
        _ => Err(EvalError::new("sign on non-numeric")),
    }
}

// ---------------------------------------------------------------------------
// String
// ---------------------------------------------------------------------------

fn fn_length(args: &[Value]) -> Result<Value, EvalError> {
    let v = single(args, "length")?;
    match v {
        Value::Text(s) => i64_from(s.chars().count()),
        Value::List(l) => i64_from(l.len()),
        Value::Map(m) => i64_from(m.len()),
        _ => Err(EvalError::new("length on unsupported type")),
    }
}

fn str_upper(args: &[Value]) -> Result<Value, EvalError> {
    match single(args, "upper")? {
        Value::Text(s) => Ok(Value::Text(s.to_uppercase())),
        _ => Err(EvalError::new("upper on non-string")),
    }
}

fn str_lower(args: &[Value]) -> Result<Value, EvalError> {
    match single(args, "lower")? {
        Value::Text(s) => Ok(Value::Text(s.to_lowercase())),
        _ => Err(EvalError::new("lower on non-string")),
    }
}

fn str_trim(args: &[Value]) -> Result<Value, EvalError> {
    match single(args, "trim")? {
        Value::Text(s) => Ok(Value::Text(s.trim().to_string())),
        _ => Err(EvalError::new("trim on non-string")),
    }
}

fn str_trim_start(args: &[Value]) -> Result<Value, EvalError> {
    match single(args, "trimStart")? {
        Value::Text(s) => Ok(Value::Text(s.trim_start().to_string())),
        _ => Err(EvalError::new("trimStart on non-string")),
    }
}

fn str_trim_end(args: &[Value]) -> Result<Value, EvalError> {
    match single(args, "trimEnd")? {
        Value::Text(s) => Ok(Value::Text(s.trim_end().to_string())),
        _ => Err(EvalError::new("trimEnd on non-string")),
    }
}

fn str_starts_with(args: &[Value]) -> Result<Value, EvalError> {
    if args.len() != 2 {
        return Err(EvalError::new("startsWith takes 2 arguments"));
    }
    match (&args[0], &args[1]) {
        (Value::Text(s), Value::Text(p)) => Ok(Value::Bool(s.starts_with(p.as_str()))),
        _ => Err(EvalError::new("startsWith type mismatch")),
    }
}

fn str_ends_with(args: &[Value]) -> Result<Value, EvalError> {
    if args.len() != 2 {
        return Err(EvalError::new("endsWith takes 2 arguments"));
    }
    match (&args[0], &args[1]) {
        (Value::Text(s), Value::Text(p)) => Ok(Value::Bool(s.ends_with(p.as_str()))),
        _ => Err(EvalError::new("endsWith type mismatch")),
    }
}

fn str_contains(args: &[Value]) -> Result<Value, EvalError> {
    if args.len() != 2 {
        return Err(EvalError::new("contains takes 2 arguments"));
    }
    match (&args[0], &args[1]) {
        (Value::Text(s), Value::Text(p)) => Ok(Value::Bool(s.contains(p.as_str()))),
        _ => Err(EvalError::new("contains type mismatch")),
    }
}

fn str_substring(args: &[Value]) -> Result<Value, EvalError> {
    if args.is_empty() || args.len() > 3 {
        return Err(EvalError::new("substring takes 1–3 arguments"));
    }
    let s = match &args[0] {
        Value::Text(s) => s,
        _ => return Err(EvalError::new("substring on non-string")),
    };
    let chars: Vec<char> = s.chars().collect();
    let start = args.get(1).and_then(as_i64).unwrap_or(0);
    let end = args
        .get(2)
        .and_then(as_i64)
        .unwrap_or_else(|| i64::try_from(chars.len()).unwrap_or(i64::MAX));
    let s = clamp_index(start, chars.len());
    let e = clamp_index(end, chars.len());
    if e <= s {
        Ok(Value::Text(String::new()))
    } else {
        Ok(Value::Text(chars[s..e].iter().collect()))
    }
}

fn str_replace(args: &[Value]) -> Result<Value, EvalError> {
    if args.len() != 3 {
        return Err(EvalError::new("replace takes 3 arguments"));
    }
    match (&args[0], &args[1], &args[2]) {
        (Value::Text(s), Value::Text(from), Value::Text(to)) => {
            Ok(Value::Text(s.replace(from, to)))
        }
        _ => Err(EvalError::new("replace type mismatch")),
    }
}

fn str_split(args: &[Value]) -> Result<Value, EvalError> {
    if args.len() != 2 {
        return Err(EvalError::new("split takes 2 arguments"));
    }
    match (&args[0], &args[1]) {
        (Value::Text(s), Value::Text(sep)) => Ok(Value::List(
            s.split(sep.as_str())
                .map(|p| Value::Text(p.to_string()))
                .collect(),
        )),
        _ => Err(EvalError::new("split type mismatch")),
    }
}

fn str_join(args: &[Value]) -> Result<Value, EvalError> {
    if args.len() != 2 {
        return Err(EvalError::new("join takes 2 arguments"));
    }
    let list = match &args[0] {
        Value::List(l) => l,
        _ => return Err(EvalError::new("join on non-array")),
    };
    let sep = match &args[1] {
        Value::Text(s) => s,
        _ => return Err(EvalError::new("join sep must be string")),
    };
    let parts: Vec<String> = list
        .iter()
        .map(|v| match v {
            Value::Text(s) => s.clone(),
            Value::Int(i) => i.to_string(),
            Value::Float(f) => f.to_string(),
            Value::Bool(b) => b.to_string(),
            Value::Null => String::new(),
            _ => format!("{v:?}"),
        })
        .collect();
    Ok(Value::Text(parts.join(sep)))
}

fn str_pad_start(args: &[Value]) -> Result<Value, EvalError> {
    pad(args, true)
}

fn str_pad_end(args: &[Value]) -> Result<Value, EvalError> {
    pad(args, false)
}

fn pad(args: &[Value], start: bool) -> Result<Value, EvalError> {
    if args.len() != 3 {
        return Err(EvalError::new("padStart/padEnd takes 3 arguments"));
    }
    let s = match &args[0] {
        Value::Text(s) => s.clone(),
        _ => return Err(EvalError::new("pad on non-string")),
    };
    let len = args
        .get(1)
        .and_then(as_i64)
        .ok_or_else(|| EvalError::new("pad len must be int"))?;
    let ch = match &args[2] {
        Value::Text(c) if !c.is_empty() => c.clone(),
        _ => return Err(EvalError::new("pad char must be string")),
    };
    let len = usize::try_from(len.max(0)).map_err(|_| EvalError::new("pad len too large"))?;
    let mut current: usize = s.chars().count();
    let mut out = s;
    while current < len {
        if start {
            let mut new_s = ch.clone();
            new_s.push_str(&out);
            out = new_s;
        } else {
            out.push_str(&ch);
        }
        current += ch.chars().count();
    }
    Ok(Value::Text(out))
}

fn str_truncate(args: &[Value]) -> Result<Value, EvalError> {
    if args.len() != 2 {
        return Err(EvalError::new("truncate takes 2 arguments"));
    }
    let s = match &args[0] {
        Value::Text(s) => s.clone(),
        _ => return Err(EvalError::new("truncate on non-string")),
    };
    let max = as_i64(&args[1]).ok_or_else(|| EvalError::new("truncate max must be int"))?;
    let max = usize::try_from(max.max(0)).map_err(|_| EvalError::new("truncate max too large"))?;
    let chars: Vec<char> = s.chars().collect();
    if chars.len() <= max {
        Ok(Value::Text(s))
    } else {
        Ok(Value::Text(chars[..max].iter().collect()))
    }
}

// ---------------------------------------------------------------------------
// Array
// ---------------------------------------------------------------------------

fn arr_first(args: &[Value]) -> Result<Value, EvalError> {
    let list = list_ref(&single(args, "first")?, "first")?.to_vec();
    Ok(list.first().cloned().unwrap_or(Value::Null))
}

fn arr_last(args: &[Value]) -> Result<Value, EvalError> {
    let list = list_ref(&single(args, "last")?, "last")?.to_vec();
    Ok(list.last().cloned().unwrap_or(Value::Null))
}

fn arr_at(args: &[Value]) -> Result<Value, EvalError> {
    if args.len() != 2 {
        return Err(EvalError::new("at takes 2 arguments"));
    }
    let list = list_ref(&args[0], "at")?;
    let i = as_i64(&args[1]).ok_or_else(|| EvalError::new("at index must be int"))?;
    let n = list.len();
    let n_i64 = i64::try_from(n).map_err(|_| EvalError::new("list length overflow"))?;
    let abs = if i < 0 { n_i64 + i } else { i };
    if abs < 0 {
        return Ok(Value::Null);
    }
    let abs = usize::try_from(abs).map_err(|_| EvalError::new("index overflow"))?;
    Ok(list.get(abs).cloned().unwrap_or(Value::Null))
}

fn array_slice_fn(args: &[Value]) -> Result<Value, EvalError> {
    if args.is_empty() {
        return Err(EvalError::new("slice requires at least 1 arg"));
    }
    array_slice(&args[0], &args[1..])
}

fn array_concat_fn(args: &[Value]) -> Result<Value, EvalError> {
    if args.is_empty() {
        return Err(EvalError::new("concat requires at least 1 arg"));
    }
    array_concat(&args[0], &args[1..])
}

fn arr_reverse(args: &[Value]) -> Result<Value, EvalError> {
    let list = list_ref(&single(args, "reverse")?, "reverse")?.to_vec();
    let mut out = list;
    out.reverse();
    Ok(Value::List(out))
}

fn arr_sort(args: &[Value]) -> Result<Value, EvalError> {
    let list = list_ref(&single(args, "sort")?, "sort")?.to_vec();
    let mut out = list;
    out.sort_by(|a, b| cmp_values(a, b).unwrap_or(std::cmp::Ordering::Equal));
    Ok(Value::List(out))
}

fn arr_unique(args: &[Value]) -> Result<Value, EvalError> {
    let list = list_ref(&single(args, "unique")?, "unique")?.to_vec();
    let mut out: Vec<Value> = Vec::new();
    for v in list {
        if !out.iter().any(|x| x == &v) {
            out.push(v);
        }
    }
    Ok(Value::List(out))
}

fn arr_flatten(args: &[Value]) -> Result<Value, EvalError> {
    let list = list_ref(&single(args, "flatten")?, "flatten")?.to_vec();
    let mut out = Vec::new();
    for v in list {
        match v {
            Value::List(l) => out.extend(l),
            other => out.push(other),
        }
    }
    Ok(Value::List(out))
}

fn arr_take(args: &[Value]) -> Result<Value, EvalError> {
    if args.len() != 2 {
        return Err(EvalError::new("take takes 2 arguments"));
    }
    let list = list_ref(&args[0], "take")?;
    let n = as_i64(&args[1]).ok_or_else(|| EvalError::new("take n must be int"))?;
    let n = usize::try_from(n.max(0)).map_err(|_| EvalError::new("take n overflow"))?;
    Ok(Value::List(list.iter().take(n).cloned().collect()))
}

fn arr_skip(args: &[Value]) -> Result<Value, EvalError> {
    if args.len() != 2 {
        return Err(EvalError::new("skip takes 2 arguments"));
    }
    let list = list_ref(&args[0], "skip")?;
    let n = as_i64(&args[1]).ok_or_else(|| EvalError::new("skip n must be int"))?;
    let n = usize::try_from(n.max(0)).map_err(|_| EvalError::new("skip n overflow"))?;
    Ok(Value::List(list.iter().skip(n).cloned().collect()))
}

// ---------------------------------------------------------------------------
// Object
// ---------------------------------------------------------------------------

fn obj_keys(args: &[Value]) -> Result<Value, EvalError> {
    let v = single(args, "keys")?;
    match v {
        Value::Map(m) => Ok(Value::List(
            m.into_keys().map(Value::Text).collect::<Vec<_>>(),
        )),
        _ => Err(EvalError::new("keys on non-map")),
    }
}

fn obj_values(args: &[Value]) -> Result<Value, EvalError> {
    let v = single(args, "values")?;
    match v {
        Value::Map(m) => Ok(Value::List(m.into_values().collect())),
        _ => Err(EvalError::new("values on non-map")),
    }
}

fn obj_entries(args: &[Value]) -> Result<Value, EvalError> {
    let v = single(args, "entries")?;
    match v {
        Value::Map(m) => Ok(Value::List(
            m.into_iter()
                .map(|(k, v)| Value::List(vec![Value::Text(k), v]))
                .collect(),
        )),
        _ => Err(EvalError::new("entries on non-map")),
    }
}

fn obj_has_key(args: &[Value]) -> Result<Value, EvalError> {
    if args.len() != 2 {
        return Err(EvalError::new("hasKey takes 2 args"));
    }
    match (&args[0], &args[1]) {
        (Value::Map(m), Value::Text(k)) => Ok(Value::Bool(m.contains_key(k))),
        _ => Err(EvalError::new("hasKey type mismatch")),
    }
}

fn obj_pick(args: &[Value]) -> Result<Value, EvalError> {
    if args.len() != 2 {
        return Err(EvalError::new("pick takes 2 args"));
    }
    let map = match &args[0] {
        Value::Map(m) => m.clone(),
        _ => return Err(EvalError::new("pick on non-map")),
    };
    let keys = match &args[1] {
        Value::List(l) => l,
        _ => return Err(EvalError::new("pick keys must be list")),
    };
    let mut out = BTreeMap::new();
    for k in keys {
        if let Value::Text(kn) = k {
            if let Some(v) = map.get(kn) {
                out.insert(kn.clone(), v.clone());
            }
        }
    }
    Ok(Value::Map(out))
}

fn obj_omit(args: &[Value]) -> Result<Value, EvalError> {
    if args.len() != 2 {
        return Err(EvalError::new("omit takes 2 args"));
    }
    let mut map = match &args[0] {
        Value::Map(m) => m.clone(),
        _ => return Err(EvalError::new("omit on non-map")),
    };
    let keys = match &args[1] {
        Value::List(l) => l,
        _ => return Err(EvalError::new("omit keys must be list")),
    };
    for k in keys {
        if let Value::Text(kn) = k {
            map.remove(kn);
        }
    }
    Ok(Value::Map(map))
}

fn obj_merge(args: &[Value]) -> Result<Value, EvalError> {
    if args.len() != 2 {
        return Err(EvalError::new("merge takes 2 args"));
    }
    let mut a = match &args[0] {
        Value::Map(m) => m.clone(),
        _ => return Err(EvalError::new("merge on non-map")),
    };
    let b = match &args[1] {
        Value::Map(m) => m.clone(),
        _ => return Err(EvalError::new("merge on non-map")),
    };
    for (k, v) in b {
        a.insert(k, v);
    }
    Ok(Value::Map(a))
}

// ---------------------------------------------------------------------------
// Coercion
// ---------------------------------------------------------------------------

fn coerce_to_number(args: &[Value]) -> Result<Value, EvalError> {
    let v = single(args, "toNumber")?;
    match v {
        Value::Int(_) | Value::Float(_) => Ok(v),
        Value::Text(s) => {
            if let Ok(i) = s.parse::<i64>() {
                Ok(Value::Int(i))
            } else if let Ok(f) = s.parse::<f64>() {
                Ok(Value::Float(f))
            } else {
                Ok(Value::Null)
            }
        }
        _ => Ok(Value::Null),
    }
}

fn coerce_to_string(args: &[Value]) -> Result<Value, EvalError> {
    let v = single(args, "toString")?;
    Ok(Value::Text(match v {
        Value::Text(s) => s,
        Value::Int(i) => i.to_string(),
        Value::Float(f) => f.to_string(),
        Value::Bool(b) => b.to_string(),
        Value::Null => "null".into(),
        other => format!("{other:?}"),
    }))
}

fn coerce_to_array(args: &[Value]) -> Result<Value, EvalError> {
    let v = single(args, "toArray")?;
    match v {
        Value::List(_) => Ok(v),
        other => Ok(Value::List(vec![other])),
    }
}

fn coerce_is_number(args: &[Value]) -> Result<Value, EvalError> {
    Ok(Value::Bool(matches!(
        single(args, "isNumber")?,
        Value::Int(_) | Value::Float(_)
    )))
}

fn coerce_is_string(args: &[Value]) -> Result<Value, EvalError> {
    Ok(Value::Bool(matches!(
        single(args, "isString")?,
        Value::Text(_)
    )))
}

fn coerce_is_array(args: &[Value]) -> Result<Value, EvalError> {
    Ok(Value::Bool(matches!(
        single(args, "isArray")?,
        Value::List(_)
    )))
}

fn coerce_is_object(args: &[Value]) -> Result<Value, EvalError> {
    Ok(Value::Bool(matches!(
        single(args, "isObject")?,
        Value::Map(_)
    )))
}

fn coerce_is_null(args: &[Value]) -> Result<Value, EvalError> {
    Ok(Value::Bool(matches!(single(args, "isNull")?, Value::Null)))
}

fn coerce_is_empty(args: &[Value]) -> Result<Value, EvalError> {
    let v = single(args, "isEmpty")?;
    let empty = match &v {
        Value::Null => true,
        Value::Text(s) => s.is_empty(),
        Value::List(l) => l.is_empty(),
        Value::Map(m) => m.is_empty(),
        _ => false,
    };
    Ok(Value::Bool(empty))
}

// ---------------------------------------------------------------------------
// Number formatting
// ---------------------------------------------------------------------------

fn fmt_number(args: &[Value]) -> Result<Value, EvalError> {
    if args.len() != 2 {
        return Err(EvalError::new("formatNumber takes 2 args"));
    }
    let n = to_f64(args[0].clone())?;
    let prec = as_i64(&args[1]).ok_or_else(|| EvalError::new("precision must be int"))?;
    let prec = usize::try_from(prec.max(0)).unwrap_or(0);
    Ok(Value::Text(format!("{n:.prec$}")))
}

fn fmt_percent(args: &[Value]) -> Result<Value, EvalError> {
    if args.len() != 2 {
        return Err(EvalError::new("formatPercent takes 2 args"));
    }
    let n = to_f64(args[0].clone())?;
    let prec = as_i64(&args[1]).ok_or_else(|| EvalError::new("precision must be int"))?;
    let prec = usize::try_from(prec.max(0)).unwrap_or(0);
    Ok(Value::Text(format!("{:.prec$}%", n * 100.0)))
}

fn fmt_currency(args: &[Value]) -> Result<Value, EvalError> {
    if args.len() != 2 {
        return Err(EvalError::new("formatCurrency takes 2 args"));
    }
    let n = to_f64(args[0].clone())?;
    let code = match &args[1] {
        Value::Text(s) => s.clone(),
        _ => return Err(EvalError::new("currency code must be string")),
    };
    Ok(Value::Text(format!("{n:.2} {code}")))
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn single(args: &[Value], name: &str) -> Result<Value, EvalError> {
    if args.len() != 1 {
        return Err(EvalError::new(format!("{name} takes exactly 1 argument")));
    }
    Ok(args[0].clone())
}

fn to_f64(v: Value) -> Result<f64, EvalError> {
    match v {
        Value::Int(i) => Ok(i as f64),
        Value::Float(f) => Ok(f),
        _ => Err(EvalError::new("expected numeric argument")),
    }
}

fn as_i64(v: &Value) -> Option<i64> {
    match v {
        Value::Int(i) => Some(*i),
        Value::Float(f) if f.fract() == 0.0 => Some(*f as i64),
        _ => None,
    }
}

fn i64_from(n: usize) -> Result<Value, EvalError> {
    let v = i64::try_from(n).map_err(|_| EvalError::new("length exceeds i64 range"))?;
    Ok(Value::Int(v))
}

fn list_ref<'a>(v: &'a Value, name: &str) -> Result<&'a [Value], EvalError> {
    match v {
        Value::List(l) => Ok(l.as_slice()),
        _ => Err(EvalError::new(format!("{name} on non-array"))),
    }
}

fn clamp_index(i: i64, len: usize) -> usize {
    let n = i64::try_from(len).unwrap_or(i64::MAX);
    let idx = if i < 0 { (n + i).max(0) } else { i.min(n) };
    usize::try_from(idx).unwrap_or(0)
}

fn flatten_one(args: &[Value]) -> Vec<Value> {
    if args.len() == 1 {
        if let Value::List(l) = &args[0] {
            return l.clone();
        }
    }
    args.to_vec()
}

fn numeric_lt(a: &Value, b: &Value) -> Result<bool, EvalError> {
    match cmp_values(a, b)? {
        std::cmp::Ordering::Less => Ok(true),
        _ => Ok(false),
    }
}

fn cmp_values(a: &Value, b: &Value) -> Result<std::cmp::Ordering, EvalError> {
    match (a, b) {
        (Value::Int(x), Value::Int(y)) => Ok(x.cmp(y)),
        (Value::Float(x), Value::Float(y)) => x
            .partial_cmp(y)
            .ok_or_else(|| EvalError::new("NaN in compare")),
        (Value::Int(x), Value::Float(y)) => (*x as f64)
            .partial_cmp(y)
            .ok_or_else(|| EvalError::new("NaN in compare")),
        (Value::Float(x), Value::Int(y)) => x
            .partial_cmp(&(*y as f64))
            .ok_or_else(|| EvalError::new("NaN in compare")),
        (Value::Text(x), Value::Text(y)) => Ok(x.cmp(y)),
        _ => Err(EvalError::new("compare type mismatch")),
    }
}
