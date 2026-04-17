# TRANSFORM Expression Grammar

**Status:** Specification. The BNF below is the positive-allowlist contract for the TRANSFORM primitive's expression language. Any token or AST shape not produced by this grammar is a parse error with `E_TRANSFORM_SYNTAX`.

**Authored:** 2026-04-15 as a pre-R3 blocker for Phase 1 (deliverable T12, per `.addl/phase-1/00-implementation-plan.md` R1 Triage Addendum §"TRANSFORM grammar").

**Audience:** The R5 implementer building `crates/benten-eval/src/expr/{parser,eval}.rs`; the R3 security test writer enumerating the rejection test matrix; developers authoring handler subgraphs.

## Phase-1 implementation notes

The Phase-1 parser accepts a small set of syntactic forms that expand the
surface beyond the strict BNF below. None of these divergences enlarges
the computed power of the language or introduces non-determinism; each
exists to support Phase-1 test fixtures or JS-familiar ergonomics.

- **Single-quoted string literals** (`'x'` equivalent to `"x"`). The
  denylist rejection tests exercise keywords inside single-quoted
  invocations (`require('x')`); the parser accepts either quote form so
  the rejection fires on the rejected keyword rather than on the quote
  style. Semantics are identical to double-quoted strings. See
  `crates/benten-eval/src/expr/parser.rs` `lex_string_single`.
- **Namespaced built-in aliases** (`Math.min(a, b)`, `Math.max(...)`,
  `Math.abs(x)`, `Math.floor(x)`, `Math.ceil(x)`, `Math.round(x)`,
  `Math.sqrt(x)`, `Math.pow(b, e)`, `Math.log(x)`, `Math.log10(x)`,
  `Math.log2(x)`, `Math.exp(x)`, `Math.sign(x)`, `Math.trunc(x)`;
  `String.lower(s)`, `String.upper(s)`; `Array.from(...)`,
  `Object.keys(o)`, `Number.toNumber(x)`). Accepted as aliases for the
  canonical bare-name forms (`min`, `max`, `abs`, …). See
  `crates/benten-eval/src/expr/eval.rs` `try_namespaced_call`.
- **JavaScript-style method calls on receivers** — `s.toLowerCase()`,
  `s.toUpperCase()`, `s.trim()`, `s.startsWith(p)`, `s.endsWith(p)`,
  `s.slice(a, b)`, `s.concat(x)`, and array methods
  `arr.map`/`.filter`/`.reduce`/`.find`/`.findIndex`/`.every`/`.some`/`.sortBy`/`.uniqueBy`/`.groupBy`/`.count`.
  Array methods additionally admit lambda arguments (the only lambda
  form the grammar accepts at all). See `dispatch_method` in
  `expr/eval.rs`.

Phase 2 may tighten the parser to match the strict grammar below if the
divergence proves costly. Mini-review findings `g6-opl-3` / `g6-opl-4`
/ `g6-cr-7` track this discussion.

## Design philosophy

- **Allowlist, not denylist.** Every production is positively defined. Any syntactic construct not listed is rejected at parse time. Adding new constructs requires a spec update + grammar version bump.
- **Pure.** No side effects. Expressions are a function of their inputs (context bindings) to their output. No I/O, no time, no RNG, no mutation of external state.
- **Deterministic.** Identical inputs produce identical outputs across processes, architectures, and Rust versions. This mirrors Benten's content-hash determinism requirement (ENGINE-SPEC §7).
- **Non-Turing.** No recursion, no loops, no closures. The 50+ array built-ins provide the iteration patterns needed without introducing unbounded computation.
- **Familiar.** Reads like JavaScript for the allowed subset. A developer familiar with JS sees accepted expressions and has the correct intuition about what they do.

## BNF (grammar version 1.0)

Notation: `::=` defines a production; `|` is alternation; `[x]` is optional; `{x}` is zero-or-more; `'literal'` is a literal token; UPPERCASE names are lexical tokens.

```
expression    ::= ternary

ternary       ::= logical_or [ '?' expression ':' expression ]

logical_or    ::= logical_and { '||' logical_and }

logical_and   ::= equality { '&&' equality }

equality      ::= comparison { ('==' | '!=' | '===' | '!==') comparison }

comparison    ::= additive { ('<' | '<=' | '>' | '>=') additive }

additive      ::= multiplicative { ('+' | '-') multiplicative }

multiplicative ::= unary { ('*' | '/' | '%') unary }

unary         ::= ('!' | '-' | '+') unary
               |  postfix

postfix       ::= primary { postfix_op }

postfix_op    ::= '.' IDENTIFIER
               |  '[' expression ']'
               |  '(' [ argument_list ] ')'

argument_list ::= expression { ',' expression }

primary       ::= NUMBER
               |  STRING
               |  'true' | 'false' | 'null' | 'undefined'
               |  IDENTIFIER
               |  '$' IDENTIFIER                   (context binding: $input, $result, $item, $index, $results, $error)
               |  array_literal
               |  object_literal
               |  '(' expression ')'
               |  builtin_call

array_literal ::= '[' [ expression { ',' expression } [ ',' ] ] ']'

object_literal ::= '{' [ property { ',' property } [ ',' ] ] '}'

property      ::= IDENTIFIER ':' expression
               |  STRING ':' expression
               |  IDENTIFIER                        (shorthand; equivalent to IDENTIFIER: IDENTIFIER)

builtin_call  ::= BUILTIN_NAME '(' [ argument_list ] ')'
```

### Lexical tokens

```
NUMBER        ::= /-?[0-9]+(\.[0-9]+)?([eE][-+]?[0-9]+)?/    (double-precision float; NaN and ±Inf are rejected)
STRING        ::= /"(?:[^"\\]|\\["\\nrt])*"/                 (double-quoted; escapes: \" \\ \n \r \t only)
IDENTIFIER    ::= /[a-zA-Z_][a-zA-Z_0-9]*/                   (ASCII only; no Unicode identifiers)
BUILTIN_NAME  ::= one of the names in the Built-ins appendix below
```

### Operator precedence (high to low)

1. `.` `[]` `()` — postfix (left-associative)
2. `!` `-` `+` — unary prefix
3. `*` `/` `%` — multiplicative (left-associative)
4. `+` `-` — additive (left-associative)
5. `<` `<=` `>` `>=` — comparison (non-associative; `a < b < c` is a parse error)
6. `==` `!=` `===` `!==` — equality (non-associative)
7. `&&` — logical AND (left-associative, short-circuit)
8. `||` — logical OR (left-associative, short-circuit)
9. `?:` — ternary (right-associative)

## Built-ins (allowlist, v1.0)

Invocable as `BUILTIN_NAME(args...)`. Not bound to any object — no `Math.min`, write `min`.

### Arithmetic

`abs`, `ceil`, `floor`, `round`, `min`, `max`, `sum`, `product`, `sqrt`, `pow(base, exp)`, `log`, `log10`, `log2`, `exp`, `sign`, `trunc`

### String

`length(s)`, `upper(s)`, `lower(s)`, `trim(s)`, `trimStart(s)`, `trimEnd(s)`,
`startsWith(s, prefix)`, `endsWith(s, suffix)`, `contains(s, substr)`,
`substring(s, start[, end])`, `replace(s, from, to)`, `split(s, sep)`, `join(arr, sep)`,
`padStart(s, len, char)`, `padEnd(s, len, char)`, `truncate(s, max)`

### Array

`length(arr)`, `first(arr)`, `last(arr)`, `at(arr, i)`, `slice(arr, start[, end])`,
`concat(a, b)`, `reverse(arr)`, `sort(arr)` (default comparator only),
`sortBy(arr, key_expr)`, `unique(arr)`, `uniqueBy(arr, key_expr)`,
`filter(arr, predicate_expr)`, `map(arr, transform_expr)`, `reduce(arr, init, reducer_expr)`,
`find(arr, predicate_expr)`, `findIndex(arr, predicate_expr)`, `every(arr, predicate_expr)`,
`some(arr, predicate_expr)`, `count(arr, predicate_expr)`, `groupBy(arr, key_expr)`,
`flatten(arr)`, `take(arr, n)`, `skip(arr, n)`

### Object

`keys(obj)`, `values(obj)`, `entries(obj)`, `hasKey(obj, key)`,
`pick(obj, keys_array)`, `omit(obj, keys_array)`, `merge(a, b)` (shallow)

### Coercion

`toNumber(s)` (returns `null` on parse failure, not `NaN`),
`toString(v)`, `toArray(v)` (wraps non-arrays into single-element arrays),
`isNumber(v)`, `isString(v)`, `isArray(v)`, `isObject(v)`, `isNull(v)`, `isEmpty(v)`

### Time (deterministic)

**TRANSFORM has no clock access.** `now()` is **not** a built-in. The HLC timestamp injected by `crud('post')` lives at `$input.createdAt`; reference it via the context binding, don't call time functions.

### Number formatting

`formatNumber(n, precision)`, `formatPercent(n, precision)`, `formatCurrency(n, currency_code)`

## Rejected constructs (denylist appendix)

All of the following produce `E_TRANSFORM_SYNTAX` at parse time. This list is informational — the *real* rejection is that these aren't in the BNF above. The list exists so R3 security writers have a concrete coverage checklist.

### Language features explicitly rejected

1. **Closures / function expressions** — `function (x) { … }`, `x => x + 1`, `(x) => { return x }`.
2. **`this`** — in any position. No implicit receiver.
3. **Imports / requires** — `import x from 'y'`, `require('x')`, `await import(…)`.
4. **Prototype access** — `obj.__proto__`, `obj.constructor`, `obj.prototype`, `Object.getPrototypeOf(…)`.
5. **Tagged template literals** — `` fn`template ${expr}` ``. The tag call is rejected.
6. **Template literals with expressions** — `` `prefix ${expr} suffix` ``. Templates are not in the BNF at all; use `"string".concat(...)` or `join(...)` instead.
7. **Optional chaining** — `obj?.method?.()`. Use `hasKey(obj, 'method')` and an explicit ternary.
8. **Computed property names** — `{ [expr]: value }`. Object literal keys must be `IDENTIFIER` or `STRING` literal, not computed.
9. **`new`** — in any position. No constructor calls.
10. **`with`** — legacy statement form; grammar has no statements at all.
11. **Destructuring with getters** — `const { [Symbol.iterator]: x } = obj`. No destructuring patterns.
12. **`eval` / `Function` / `new Function`** — runtime code compilation.
13. **`yield` / `async` / `await`** — coroutines / promises.
14. **`Symbol.*` / `Reflect.*` / `Proxy`** — meta-programming.
15. **Spread in call position** — `fn(...args)`. Spread is not in the BNF.
16. **Comma operator** — `(a, b)`. Parentheses group a single expression; comma is only an arg separator.
17. **`instanceof` / `typeof` / `in`** — not in the BNF. Use `isNumber(v)` etc. for type checks.
18. **Regex literals** — `/pattern/flags`. No regex support in v1.0; grammar rejects the `/` start.
19. **Bitwise operators** — `&` `|` `^` `~` `<<` `>>` `>>>`. Not in the BNF.
20. **Assignment operators** — `=` `+=` `-=` `*=` `/=` `%=`. No mutation; expressions are pure.
21. **Increment / decrement** — `++` `--`. Same reason.
22. **Exponentiation** — `**`. Use `pow(base, exp)`.
23. **Nullish coalescing** — `a ?? b`. Use `isNull(a) ? b : a`.
24. **Labeled statements / break / continue / return / throw** — no statements.
25. **`delete`** — no mutation.

## Rejection test matrix (R3 security-writer guidance)

Every class above gets a named test in `crates/benten-eval/tests/transform_grammar_rejects_*.rs`:

- `transform_grammar_rejects_closure_arrow`
- `transform_grammar_rejects_closure_function_expression`
- `transform_grammar_rejects_this_keyword`
- `transform_grammar_rejects_import_statement`
- `transform_grammar_rejects_require_call`
- `transform_grammar_rejects_proto_access`
- `transform_grammar_rejects_constructor_access`
- `transform_grammar_rejects_prototype_access`
- `transform_grammar_rejects_tagged_template`
- `transform_grammar_rejects_template_literal_with_expression`
- `transform_grammar_rejects_optional_chaining`
- `transform_grammar_rejects_computed_property_name`
- `transform_grammar_rejects_new_keyword`
- `transform_grammar_rejects_with_statement`
- `transform_grammar_rejects_destructuring_pattern`
- `transform_grammar_rejects_eval_call`
- `transform_grammar_rejects_new_function`
- `transform_grammar_rejects_yield`
- `transform_grammar_rejects_async_await`
- `transform_grammar_rejects_symbol_access`
- `transform_grammar_rejects_reflect_access`
- `transform_grammar_rejects_proxy_construction`
- `transform_grammar_rejects_spread_in_call`
- `transform_grammar_rejects_comma_operator`
- `transform_grammar_rejects_instanceof`
- `transform_grammar_rejects_typeof`
- `transform_grammar_rejects_in_operator`
- `transform_grammar_rejects_regex_literal`
- `transform_grammar_rejects_bitwise_and`
- `transform_grammar_rejects_bitwise_or`
- `transform_grammar_rejects_bitwise_xor`
- `transform_grammar_rejects_bitshift`
- `transform_grammar_rejects_assignment`
- `transform_grammar_rejects_increment`
- `transform_grammar_rejects_exponent`
- `transform_grammar_rejects_nullish_coalescing`
- `transform_grammar_rejects_return_statement`
- `transform_grammar_rejects_throw_statement`
- `transform_grammar_rejects_delete_statement`

Each test calls `parse("<expression>")` and asserts `Err(E_TRANSFORM_SYNTAX)` with an `offset` matching the first rejected token.

## Fuzz harness (R3 performance-writer / security-writer)

A criterion-adjacent fuzz harness runs generated JS-like snippets through the TRANSFORM parser and asserts two properties:

1. Every accepted string evaluates deterministically (no I/O, no prototype touches, no wall-clock, no RNG) across two runs with identical inputs.
2. No input causes a panic. Every rejected input produces `E_TRANSFORM_SYNTAX` cleanly.

Harness location: `crates/benten-eval/tests/fuzz_transform_parser.rs` (marked `#[ignore]` for `cargo test`; run via `cargo test -- --ignored` or in a dedicated CI job).

## Error format

`E_TRANSFORM_SYNTAX` context fields:

```
{
  reason: string,           // Human-readable: "unexpected token `new`"
  offset: number,           // Byte offset in the expression string
  expression: string,       // The expression that failed
  grammar_doc: string       // Pointer to this doc: "docs/TRANSFORM-GRAMMAR.md"
}
```

See `docs/ERROR-CATALOG.md` for the full entry.

## Versioning

This grammar is **v1.0**. Adding a construct (e.g., a new built-in) is a minor version bump. Removing a construct or changing semantics is a major version bump and requires a migration path. The grammar version ships in every compiled engine; subgraphs carry their grammar version so Phase 2 can detect drift.

## Open questions (deferred)

- **Regex support.** Deferred to post-Phase-1. A deterministic regex subset (no backtracking, bounded-time) could be added in v1.1.
- **BigInt / i128 / u64.** Deferred. All numbers are JS-like double-precision floats in v1.0.
- **Private state / memoization.** Rejected by design. Expressions are pure and stateless.
- **User-defined built-ins.** Deferred. Users who need custom logic route through SANDBOX (Phase 2) or CALL to another subgraph.
