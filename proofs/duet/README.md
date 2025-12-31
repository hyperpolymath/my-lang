# Duet Dialect: Formal Specification

The **Duet** dialect enables collaborative human-AI pair programming through
session types, two-party protocols, and neurosymbolic extensions.

## Contents

1. [Protocol Theory](protocol-theory/) - Two-party communication protocols
2. [Session Types](session-types/) - Typed communication channels
3. [Collaboration Semantics](collaboration/) - Human-AI interaction model

## Overview

Duet introduces collaborative programming concepts:

- **Session types** for typed bidirectional communication
- **Two-party protocols** between human and AI
- **Neurosymbolic extensions** combining neural and symbolic reasoning
- **Balanced co-creation** where neither party dominates

## Design Philosophy

### The Duet Model

Programming as musical duet:
- **Human**: Provides intent, domain knowledge, creativity
- **AI**: Provides execution, patterns, optimization
- **Protocol**: Ensures harmonious interaction

### Communication Patterns

```
Human → AI:  Requests, specifications, corrections
AI → Human:  Suggestions, completions, explanations
Bidirectional: Negotiation, refinement, validation
```

## Key Theorems

- **Theorem 1 (Session Fidelity)**: Communication follows the declared protocol
- **Theorem 2 (Deadlock Freedom)**: Well-typed sessions don't deadlock
- **Theorem 3 (Progress)**: Sessions make progress until completion
- **Theorem 4 (Safety)**: No protocol violations occur

## Relationship to Other Dialects

- **From Solo**: Adds two-party interaction, session types
- **To Ensemble**: Enables multi-party orchestration
