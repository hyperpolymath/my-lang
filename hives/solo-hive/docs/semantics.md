# Solo Semantics (Operational Outline)

This is a minimal, implementable semantics for the Solo core. It is intended
as a guide for the interpreter and for later formalization.

## Judgments

- Expression evaluation: `env, store |- e => v, store'`
- Statement execution: `env, store |- s => env', store', result`
- Block execution: `env, store |- { s* } => env', store', result`

`result` is one of:
- `Unit`
- `Return(v)`
- `Error(msg)`

## Values

- `Int(n)`, `Bool(b)`, `String(s)`
- `Record(map)`, `Array(list)`
- `Function(closure)`
- `EffectHandle(name)` (placeholder; actual effect semantics are runtime-defined)

## Core Rules (informal)

### Variables
- `env(x) = v` implies `env, store |- x => v, store`

### Let binding
- Evaluate RHS first, then bind name in the current environment.
- `mut` allows rebinding in the same scope.

### If/Else
- Evaluate condition; if true, evaluate then-block; else evaluate else-block.

### Return
- Evaluate expression and wrap as `Return(v)`.
- A `Return` short-circuits the nearest function body/block.

### Call
- Evaluate callee and args left-to-right.
- If callee is a function value, create a new environment with parameters bound
  to arg values, then evaluate the function body.

### Field access
- Evaluate target expression; if it is a record, return the named field.

### Binary operators
- Evaluate left, then right.
- Numeric ops require `Int` (or numeric types if extended).
- `==`, `!=`, `<`, `>` are defined for `Int` and `Bool` (and `String` if allowed).

### try / ?
- `try e` propagates error handling policy. Minimal behavior: evaluate `e` and
  return its value; if `e` raises `Error`, propagate it unchanged.
- `e?` is sugar for `try e` in statement position.

### restrict
- Semantically a no-op in core; used by later dialects or the type system.

### comptime
- Execute block at compile-time in a separate interpreter context. If not
  available, treat as normal block evaluation and mark as TODO.

### effects
- `effect` declarations introduce a named effect interface; runtime semantics
  are implementation-defined. Parsing stores effect signatures in the AST.

## Scoping

- Blocks introduce a new environment frame.
- Function bodies introduce a new environment frame.
- `let` binds in the current frame.

## Error Handling

- Errors are explicit results, not panics.
- Parsing errors and runtime errors should include source spans.
