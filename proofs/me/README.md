# Me Dialect: Formal Specification

The **Me** dialect is My Language's visual, block-based programming interface
designed for learners ages 8-12. This directory contains formal specifications
for the visual semantics, block calculus, and pedagogical foundations.

## Contents

1. [Visual Semantics](visual-semantics/) - Formal meaning of visual constructs
2. [Block Calculus](block-calculus/) - Algebraic theory of block composition
3. [Pedagogy](pedagogy/) - Educational theory and learning progression

## Overview

Me introduces programming concepts through:

- **Drag-and-drop blocks** representing program constructs
- **Colored tokens** representing resources (affine types)
- **"Use once" rules** teaching resource management
- **Visual dataflow** showing information movement

## Design Principles

### 1. Progressive Disclosure

Concepts are revealed gradually:

| Level | Concepts | Visual Elements |
|-------|----------|-----------------|
| 1 | Sequence, output | Print blocks, linear chains |
| 2 | Variables, input | Value blocks, connectors |
| 3 | Conditionals | Branch blocks (if/else) |
| 4 | Loops | Repeat blocks |
| 5 | Functions | Definition blocks, call blocks |
| 6 | Resources | Colored tokens, "use once" rules |

### 2. Concrete Before Abstract

- Physical manipulation (blocks) before symbolic representation (text)
- Visual types (colors) before named types
- Immediate feedback before deferred errors

### 3. Safe Failure

- No syntax errors (only valid connections allowed)
- Type errors shown visually (blocks don't fit)
- Resource errors shown through token tracking

## Key Theorems

- **Theorem 1 (Visual Soundness)**: Valid block arrangements correspond to well-typed programs
- **Theorem 2 (Block Equivalence)**: Semantically equivalent blocks have the same denotation
- **Theorem 3 (Learning Progression)**: Pedagogical ordering aligns with cognitive development
- **Theorem 4 (Resource Visualization)**: Token tracking correctly models affine types

## Relationship to Other Dialects

Me programs can be **lifted** to Solo programs:

```
lift : MeProgram → SoloProgram
```

This lifting preserves semantics while revealing textual syntax.

## Research Questions

1. Does visual affine typing improve resource management understanding?
2. How does block-based programming transfer to textual coding?
3. What is the optimal visual vocabulary for type concepts?
