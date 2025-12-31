# AI Semantics

This directory contains the formal semantics for My Language's first-class AI integration.

## Contents

1. [AI Type Theory](type-theory.md) - Typing rules for AI constructs
2. [AI Operational Semantics](operational.md) - Reduction rules for AI operations
3. [AI Contracts](contracts.md) - AI-aware contract verification
4. [AI Effect Handlers](handlers.md) - Effect handler semantics for AI
5. [Prompt Semantics](prompts.md) - Prompt template formalization

## Overview

My Language treats AI as a first-class computational effect, enabling:

- **Typed AI Operations**: AI calls return typed results
- **Effect Tracking**: AI operations are tracked in the type system
- **Contract Integration**: AI verification in pre/post conditions
- **Prompt Templates**: First-class, typed prompt definitions

## AI as Effect

AI operations are modeled as effects in the type system:

```
Γ ⊢ ai query { prompt: e } : AI⟨String⟩ ! AI
```

The `AI` effect indicates that this computation requires AI capabilities.

## Key Theorems

- **Theorem 1 (AI Type Safety)**: AI operations respect declared types
- **Theorem 2 (AI Effect Soundness)**: AI effects are properly tracked
- **Theorem 3 (Prompt Type Safety)**: Prompt invocations preserve types
- **Theorem 4 (Contract Verification)**: AI constraints are honored
