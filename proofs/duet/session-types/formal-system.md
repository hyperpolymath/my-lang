# Session Types Formal System

## Overview

Session types provide a type discipline for communication protocols,
ensuring that interacting parties follow an agreed-upon protocol.

## Session Type Syntax

### Types

```
Session Type:
S ::= !τ.S                      -- Send type τ, continue as S
    | ?τ.S                      -- Receive type τ, continue as S
    | S ⊕ S                     -- Internal choice (sender chooses)
    | S & S                     -- External choice (receiver chooses)
    | μα.S                      -- Recursive session
    | α                         -- Session variable
    | end                       -- Session termination
    | S ⊗ S                     -- Parallel composition
    | S ⊸ S                     -- Session function

Labeled Choice:
S ⊕ {l₁: S₁, ..., lₙ: Sₙ}      -- Choose label lᵢ, continue as Sᵢ
S & {l₁: S₁, ..., lₙ: Sₙ}      -- Offer labels, continue based on choice

AI-Specific Extensions:
S ::= ...
    | ai_query(p).S             -- AI query with prompt p
    | ai_suggest(τ).S           -- AI suggests value of type τ
    | human_confirm.S           -- Human confirmation required
    | human_edit(τ).S           -- Human can edit value
```

### Duality

Session types come in dual pairs (client/server):

```
dual(!τ.S) = ?τ.dual(S)
dual(?τ.S) = !τ.dual(S)
dual(S₁ ⊕ S₂) = dual(S₁) & dual(S₂)
dual(S₁ & S₂) = dual(S₁) ⊕ dual(S₂)
dual(end) = end
dual(μα.S) = μα.dual(S)
dual(α) = α
```

## Typing Rules

### Channel Typing

```
Channel Context:
Δ ::= ∅ | Δ, c: S

Process Judgment:
Δ ⊢ P                          -- Process P uses channels in Δ
```

### Basic Communication

```
Δ, c: !τ.S ⊢ e : τ
────────────────────────────── (T-Send)
Δ, c: !τ.S ⊢ c!e; P ⊣ Δ, c: S

Δ, c: ?τ.S, x: τ ⊢ P
──────────────────────────────── (T-Receive)
Δ, c: ?τ.S ⊢ c?(x); P ⊣ Δ, c: S
```

### Choice

```
Δ, c: Sᵢ ⊢ P    i ∈ {1, ..., n}
────────────────────────────────────────── (T-Select)
Δ, c: ⊕{l₁: S₁, ..., lₙ: Sₙ} ⊢ c ◁ lᵢ; P

∀i ∈ {1,...,n}. Δ, c: Sᵢ ⊢ Pᵢ
────────────────────────────────────────────────────── (T-Branch)
Δ, c: &{l₁: S₁, ..., lₙ: Sₙ} ⊢ c ▷ {l₁: P₁, ..., lₙ: Pₙ}
```

### Session Initiation

```
Δ₁ ⊢ P    Δ₂ ⊢ Q    Δ₁(c) = dual(Δ₂(c))
─────────────────────────────────────────── (T-Session)
Δ₁ \ c, Δ₂ \ c ⊢ (νc)(P | Q)
```

### Termination

```
──────────────── (T-End)
c: end ⊢ close(c)
```

## Human-AI Collaboration Types

### AI Query Session

```
// AI asks for query parameters, human provides, AI responds
AIQuerySession = ?String.           // AI receives prompt
                 ai_query.          // AI processes
                 !String.           // AI sends response
                 human_confirm.     // Human confirms or rejects
                 end
```

### Code Assistance Session

```
CodeAssistSession = μα.
    &{ request:  ?CodeContext.      // Human sends context
                 ai_suggest(Code).  // AI suggests code
                 human_edit(Code).  // Human may edit
                 !Code.             // Final code returned
                 α,                 // Continue session

       explain:  ?Code.             // Human sends code
                 !Explanation.      // AI explains
                 α,

       done:     end
    }
```

### Pair Programming Session

```
PairProgramSession = μα.
    ⊕{ human_leads:                 // Human takes lead
         !Intent.                   // Human states intent
         ?Suggestion.               // AI suggests implementation
         ⊕{ accept: !Code. α,
            modify: human_edit(Code). !Code. α,
            reject: α
          },

       ai_leads:                    // AI takes lead
         ?Context.                  // AI analyzes context
         !Proposal.                 // AI proposes action
         &{ confirm: ?Code. α,
            refine: !Feedback. α,
            cancel: α
          },

       end_session: end
    }
```

## Operational Semantics

### Process Syntax

```
Process:
P, Q ::= c!e; P                     -- Send
       | c?(x); P                   -- Receive
       | c ◁ l; P                   -- Select
       | c ▷ {l₁: P₁, ..., lₙ: Pₙ} -- Branch
       | (νc)(P | Q)                -- Session
       | P | Q                      -- Parallel
       | 0                          -- Inaction
       | close(c)                   -- Close
       | *P                         -- Replication
```

### Reduction Rules

```
c!v; P | c?(x); Q ⟶ P | Q[v/x]                    (R-Comm)

c ◁ lᵢ; P | c ▷ {l₁: Q₁, ..., lₙ: Qₙ} ⟶ P | Qᵢ  (R-Choice)

close(c) | close(c̄) ⟶ 0                          (R-Close)

P ⟶ P'
──────────────── (R-Context)
E[P] ⟶ E[P']

P ≡ P'    P' ⟶ Q'    Q' ≡ Q
─────────────────────────────── (R-Struct)
P ⟶ Q
```

### Structural Congruence

```
P | 0 ≡ P
P | Q ≡ Q | P
(P | Q) | R ≡ P | (Q | R)
(νc)(P | Q) ≡ (νc)P | Q    if c ∉ fn(Q)
(νc)(νd)P ≡ (νd)(νc)P
*P ≡ P | *P
```

## Safety Theorems

### Theorem 1 (Session Fidelity)

Well-typed processes respect their session types.

**Proof**: By induction on typing derivation. Each communication
rule corresponds to the expected session type transition. □

### Theorem 2 (Deadlock Freedom)

Well-typed sessions in the core calculus are deadlock-free.

**Proof**: Duality ensures that for every send there's a matching
receive, and for every select there's a matching branch. The
linear use of channels prevents resource conflicts. □

### Theorem 3 (Progress)

If Δ ⊢ P and P is not 0, then either P can reduce or P is
waiting on an external input.

**Proof**: By case analysis on P. Each constructor has a matching
reduction rule or is waiting for a dual action. □

### Theorem 4 (Type Safety)

Well-typed processes don't get stuck on protocol errors.

**Proof**: Combination of Session Fidelity and Progress. □

## Multiparty Session Types

For Ensemble dialect, extend to multiple parties:

```
Global Type (Choreography):
G ::= p → q: τ.G                    -- p sends τ to q, continue as G
    | p → q: {l₁: G₁, ..., lₙ: Gₙ}  -- p chooses, q branches
    | μα.G | α | end

Projection:
G ↾ p                               -- Local type for participant p

(p → q: τ.G) ↾ p = !τ.(G ↾ p)
(p → q: τ.G) ↾ q = ?τ.(G ↾ q)
(p → q: τ.G) ↾ r = G ↾ r            (r ≠ p, q)
```

## Implementation

### Session Type Annotations

```ml
session type CodeAssist = {
    request: recv CodeContext,
             send Suggestion,
             recv Decision,
             loop,
    done: end
}

fn code_assistant(chan: Chan<CodeAssist>) {
    loop {
        match chan.offer() {
            request(ctx) => {
                let suggestion = ai generate { context: ctx };
                chan.send(suggestion);
                let decision = chan.recv();
                if decision.accepted {
                    apply(suggestion);
                }
            }
            done => {
                chan.close();
                return;
            }
        }
    }
}
```

### Runtime Enforcement

```rust
struct TypedChannel<S: SessionType> {
    channel: RawChannel,
    _phantom: PhantomData<S>,
}

impl<T, S> TypedChannel<Send<T, S>> {
    fn send(self, value: T) -> TypedChannel<S> {
        self.channel.send(value);
        TypedChannel { channel: self.channel, _phantom: PhantomData }
    }
}

impl<T, S> TypedChannel<Recv<T, S>> {
    fn recv(self) -> (T, TypedChannel<S>) {
        let value = self.channel.recv();
        (value, TypedChannel { channel: self.channel, _phantom: PhantomData })
    }
}
```
