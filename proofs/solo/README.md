# Solo Dialect: Formal Specification

The **Solo** dialect is My Language's text-based programming mode for learners
ages 13+, featuring explicit affine types, contracts, and parsing operators.

## Contents

1. [Affine Types](affine-types/) - Linear and affine type system
2. [Contracts](contracts/) - Design-by-contract verification
3. [Parsing Theory](parsing-theory/) - Cut operator and deterministic parsing

## Overview

Solo introduces advanced programming concepts:

- **Explicit affine/linear types** for resource management
- **Checkpoint/rollback** for reversible computation
- **Cut operators** for committed choice in parsing
- **Design-by-contract** with pre/post conditions
- **Human-first, AI-assists** collaboration model

## Key Features

### Affine Type System

Resources that can be used at most once:

```ml
// File handle must be used exactly once
fn process_file(handle: FileHandle¹) -> Result {
    let content = read(handle);  // handle consumed
    // handle can no longer be used
    parse(content)
}
```

### Linear Logic Foundation

Solo's type system is grounded in linear logic:

| Linear Connective | Solo Type |
|------------------|-----------|
| A ⊗ B | (A, B) tensor product |
| A ⊕ B | Either A B |
| A ⊸ B | A → B linear function |
| !A | Box⟨A⟩ unrestricted |
| ?A | Lazy⟨A⟩ on-demand |

### Contracts

Design-by-contract verification:

```ml
fn sqrt(x: Float) -> Float
where
    pre: x >= 0.0,
    post: result * result ≈ x,
    ai_check: "result is mathematically correct"
{
    // implementation
}
```

### Parsing with Cut

Deterministic parsing with committed choice:

```ml
parser expr() -> Expr {
    term() >> (
        '+' >> ! >> term() >> |t| Add(_, t)
      | '-' >> ! >> term() >> |t| Sub(_, t)
      | pure(identity)
    )
}
```

The `!` operator commits to the current alternative.

## Key Theorems

- **Theorem 1 (Affine Type Safety)**: Resources are not used after consumption
- **Theorem 2 (Linear Soundness)**: Linear resources are used exactly once
- **Theorem 3 (Contract Validity)**: Contracts are correctly enforced
- **Theorem 4 (Cut Correctness)**: Cut does not affect parsing semantics
- **Theorem 5 (Checkpoint Safety)**: Rollback restores consistent state

## Relationship to Other Dialects

- **From Me**: Adds textual syntax, explicit types, contracts
- **To Duet**: Enables two-party collaboration protocols
