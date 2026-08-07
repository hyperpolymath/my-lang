<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->
<!-- SPDX-FileCopyrightText: 2026 Jonathan D.A. Jewell <j.d.a.jewell@open.ac.uk> -->

# Solo Typing (Core Rules)

This defines the Solo core type system over the grammar in `syntax.md`.

## Judgments

- Expression typing: `Gamma |- e : T`
- Statement typing: `Gamma |- s : T` (where statements are expressions with Unit)
- Block typing: `Gamma |- { s* } : T`

`Gamma` maps identifiers to types.

## Types

```
T ::= Int | Bool | String
    | T -> T
    | Effect<T>
    | &T | &mut T
    | [T]
    | { l1: T1, ..., ln: Tn }
    | Ident (named type)
```

## Core Rules (informal)

### Variables
- If `x : T` in `Gamma`, then `Gamma |- x : T`.

### Let binding
- If `Gamma |- e : T`, then `Gamma, x:T |- let x = e : Unit`.
- `mut` allows later assignment to `x` with the same type.

### If/Else
- If `Gamma |- c : Bool`, `Gamma |- t : T`, and `Gamma |- e : T`,
  then `Gamma |- if c t else e : T`.

### Return
- If `Gamma |- e : T`, then `Gamma |- return e : T` within function scope.
  The function body must have a consistent return type.

### Call
- If `Gamma |- f : (T1, ..., Tn) -> T` and `Gamma |- ai : Ti` for all args,
  then `Gamma |- f(a1,...,an) : T`.

### Field access
- If `Gamma |- r : { ... , l:T, ... }`, then `Gamma |- r.l : T`.

### Binary operators
- `+ - * /` require `Int` (extendable to Float later).
- `== != < >` are defined for `Int` (and possibly `Bool/String` if allowed).
- `&& ||` require `Bool`.

### Block
- A block type is the type of its last expression, or `Unit` if empty.
- A `return` inside a block fixes the block to the function return type.

### Effect types
- `Effect<T>` denotes computations with effects returning `T`.
- Core typing is structural; effect checking is deferred unless implemented.

### Contracts
- `pre`, `post`, `invariant` expressions must have type `Bool`.

### Structs
- `struct Name { l1:T1, ... }` introduces a named record type `Name`.

## Notes

- This core is intentionally minimal.
- Affine/linear typing is not enforced here; add later as a dialect extension.
