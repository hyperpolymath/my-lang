# Agent Calculus: π-Calculus Foundation

## Overview

The Ensemble agent calculus is based on the π-calculus, extended with
agent-specific primitives for AI coordination.

## Syntax

### Process Syntax

```
Process:
P, Q ::= 0                          -- Inaction
       | x̄⟨ỹ⟩.P                     -- Output (send ỹ on x)
       | x(ỹ).P                     -- Input (receive ỹ on x)
       | P | Q                      -- Parallel composition
       | (νx)P                      -- Restriction (new name)
       | !P                         -- Replication
       | [x = y]P                   -- Match
       | [x ≠ y]P                   -- Mismatch
       | A⟨ỹ⟩                       -- Agent invocation

Agent-Specific Extensions:
       | agent⟨n, σ⟩.P              -- Agent instantiation (name n, spectrum σ)
       | delegate(A, task).P        -- Task delegation
       | await(A, result).P         -- Result waiting
       | orchestrate(G).P           -- Orchestration goal G
       | refine(human, x).P         -- Human refinement
```

### Names and Channels

```
Names:
x, y, z                             -- Channel names
a, b, c                             -- Value names
A, B, C                             -- Agent names

Spectrum:
σ ∈ {Red, Orange, Yellow, Green, Blue, Indigo, Violet}
```

## Operational Semantics

### Structural Congruence

```
P | 0 ≡ P                           (Par-Zero)
P | Q ≡ Q | P                       (Par-Comm)
(P | Q) | R ≡ P | (Q | R)          (Par-Assoc)
(νx)0 ≡ 0                           (Res-Zero)
(νx)(νy)P ≡ (νy)(νx)P              (Res-Comm)
(νx)(P | Q) ≡ P | (νx)Q  if x ∉ fn(P)  (Scope)
!P ≡ P | !P                         (Repl)
```

### Reduction Rules

```
Communication:
x̄⟨ỹ⟩.P | x(z̃).Q ⟶ P | Q[ỹ/z̃]       (R-Comm)

Match:
[x = x]P ⟶ P                        (R-Match)

Context:
P ⟶ P'
────────────                        (R-Par)
P | Q ⟶ P' | Q

P ⟶ P'
────────────                        (R-Res)
(νx)P ⟶ (νx)P'

P ≡ P'   P' ⟶ Q'   Q' ≡ Q
──────────────────────────          (R-Struct)
P ⟶ Q
```

### Agent-Specific Reductions

```
Agent Instantiation:
agent⟨n, σ⟩.P ⟶ (νa)(A_σ(a) | P[a/n])    (R-Agent)
    where A_σ is the agent process for spectrum σ

Delegation:
delegate(A, task).P | A(x).Q ⟶ P | Q[task/x]  (R-Delegate)

Await:
await(A, result).P | Ā⟨v⟩.Q ⟶ P[v/result] | Q  (R-Await)

Orchestration:
orchestrate(G).P ⟶ expand(G) | P       (R-Orchestrate)
    where expand(G) produces agent processes

Human Refinement:
refine(human, x).P ⟶ P[v/x]            (R-Refine)
    where v is human-provided value
```

## Agent Definitions

### Spectrum Agent Processes

```
Red Agent (Performance):
A_Red(chan) =
    !chan(task).
    let optimized = optimize(task) in
    chan̄⟨optimized⟩.0

Orange Agent (Concurrency):
A_Orange(chan) =
    !chan(task).
    go { execute(task) } |
    chan(result).
    chan̄⟨result⟩.0

Yellow Agent (Contracts):
A_Yellow(chan) =
    !chan(code).
    let checked = verify_contracts(code) in
    [checked = valid]chan̄⟨code⟩.0 +
    [checked = invalid]chan̄⟨error(checked)⟩.0

Green Agent (Config):
A_Green(chan) =
    !chan(schema).
    let config = generate_config(schema) in
    chan̄⟨config⟩.0

Blue Agent (Audit):
A_Blue(chan) =
    !chan(event).
    log(event) |
    checkpoint(event) |
    chan̄⟨ack⟩.0

Indigo Agent (Comptime):
A_Indigo(chan) =
    !chan(expr).
    let result = compile_time_eval(expr) in
    chan̄⟨result⟩.0

Violet Agent (Governance):
A_Violet(chan) =
    !chan(policy).
    let mode = parse_policy(policy) in
    set_mode(mode) |
    chan̄⟨ack⟩.0
```

## Typing

### Agent Types

```
Agent Type:
A ::= Agent⟨σ, τ_in, τ_out⟩

where:
  σ = spectrum color
  τ_in = input type
  τ_out = output type

Type Rules:

Γ ⊢ task : τ_in    A : Agent⟨σ, τ_in, τ_out⟩
────────────────────────────────────────────── (T-Delegate)
Γ ⊢ delegate(A, task) : Process

Γ, result: τ_out ⊢ P    A : Agent⟨σ, τ_in, τ_out⟩
──────────────────────────────────────────────── (T-Await)
Γ ⊢ await(A, result).P : Process
```

### Spectrum Typing

```
Red    : Agent⟨Red, Code, OptimizedCode⟩
Orange : Agent⟨Orange, Task, AsyncHandle⟩
Yellow : Agent⟨Yellow, Code, VerificationResult⟩
Green  : Agent⟨Green, Schema, Config⟩
Blue   : Agent⟨Blue, Event, Ack⟩
Indigo : Agent⟨Indigo, ComptimeExpr, Value⟩
Violet : Agent⟨Violet, Policy, Ack⟩
```

## Bisimulation

### Strong Bisimulation

Processes P and Q are strongly bisimilar (P ~ Q) if there exists a
symmetric relation R such that P R Q and:

For all P' such that P ⟶ P', there exists Q' such that Q ⟶ Q' and P' R Q'.

### Weak Bisimulation

Processes P and Q are weakly bisimilar (P ≈ Q) if there exists a
symmetric relation R such that P R Q and:

For all P' such that P ⟶ P', there exists Q' such that Q ⟹ Q' and P' R Q'.

(where ⟹ is the reflexive-transitive closure of ⟶)

### Agent Equivalence

Two agent configurations are equivalent if their observable behaviors
(inputs/outputs) are weakly bisimilar:

```
A₁ ≈_agent A₂ ⟺ ∀ctx. ctx[A₁] ≈ ctx[A₂]
```

## Safety Properties

### Theorem 1 (Agent Type Safety)

Well-typed agent processes don't get stuck on type errors.

**Proof**: By subject reduction. If Γ ⊢ P and P ⟶ P', then Γ' ⊢ P'
for some Γ' ⊇ Γ. □

### Theorem 2 (Delegation Safety)

Tasks delegated to an agent of spectrum σ are handled by an agent
with the capabilities of σ.

**Proof**: Agent instantiation creates the appropriate spectrum
process, and delegation typing ensures type compatibility. □

### Theorem 3 (No Orphan Tasks)

Every delegated task eventually receives a response (assuming agents
don't diverge).

**Proof**: Agent processes are replicated (!P) and always produce
output after receiving input. Await is matched by agent output. □

## Composition Patterns

### Sequential Composition

```
task₁ ▷ Red ▷ task₂ ▷ Yellow ▷ result

Translates to:
delegate(Red, task₁).
await(Red, t₂).
delegate(Yellow, t₂).
await(Yellow, result).P
```

### Parallel Composition

```
task₁ |▷ Red, task₂ |▷ Orange ▷ combine

Translates to:
(delegate(Red, task₁) | delegate(Orange, task₂)).
(await(Red, r₁) | await(Orange, r₂)).
let result = combine(r₁, r₂) in P
```

### Pipeline

```
input ▷ Red ▷ Yellow ▷ Blue ▷ output

Translates to:
delegate(Red, input).
await(Red, r₁).
delegate(Yellow, r₁).
await(Yellow, r₂).
delegate(Blue, r₂).
await(Blue, output).P
```
