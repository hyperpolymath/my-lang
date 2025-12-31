# Ownership Semantics

## Formal Model

### Ownership Types

We extend types with ownership qualifiers:

```
Ownership Qualifier:
q ::= own      -- Owned value
    | borrow   -- Immutably borrowed
    | mut      -- Mutably borrowed

Qualified Type:
T^q ::= τ^own | τ^borrow | τ^mut

Reference Type:
&'a τ       -- Immutable reference with lifetime 'a
&'a mut τ   -- Mutable reference with lifetime 'a
```

### Ownership Environment

```
Ownership Environment:
Ω ::= ∅
    | Ω, x: τ^own
    | Ω, x: &'a τ
    | Ω, x: &'a mut τ
    | Ω, x: moved      -- x has been moved
```

## Ownership Typing Rules

### Variable Access

```
x: τ^own ∈ Ω    x ∉ moved(Ω)
────────────────────────────── (Own-Var)
Ω ⊢ x : τ

x: &'a τ ∈ Ω    'a live in Ω
─────────────────────────────── (Borrow-Var)
Ω ⊢ x : &'a τ
```

### Move Semantics

```
Ω ⊢ e : τ^own    y fresh
──────────────────────────────────────── (Own-Let-Move)
Ω ⊢ let y = e in e' : σ ⊣ Ω[x ↦ moved], y: τ^own
```

### Copy Semantics (for Copy types)

```
τ : Copy    Ω ⊢ e : τ^own
────────────────────────────────────────── (Own-Let-Copy)
Ω ⊢ let y = e in e' : σ ⊣ Ω, y: τ^own
```

### Reference Creation

```
Ω ⊢ e : τ^own    'a fresh
──────────────────────────────── (Own-Borrow)
Ω ⊢ &e : &'a τ ⊣ Ω ∪ {x: borrowed('a)}

Ω ⊢ e : τ^own    e is lvalue    'a fresh
────────────────────────────────────────── (Own-BorrowMut)
Ω ⊢ &mut e : &'a mut τ ⊣ Ω ∪ {x: mut_borrowed('a)}
```

### Dereference

```
Ω ⊢ e : &'a τ
─────────────────── (Own-Deref)
Ω ⊢ *e : τ^borrow

Ω ⊢ e : &'a mut τ
────────────────────── (Own-DerefMut)
Ω ⊢ *e : τ^mut
```

## Move Checking

### Definition: Moved Variables

```
moved(Ω) = {x | x: moved ∈ Ω}
```

### Move Check

Before using a variable, verify it hasn't been moved:

```
fn check_move(x, Ω):
    if x ∈ moved(Ω):
        error "use of moved value"
    if x: τ^own ∈ Ω and τ ∉ Copy:
        Ω' = Ω[x ↦ moved]
        return (x, Ω')
    else:
        return (x, Ω)
```

## Ownership Invariants

### Invariant 1: Single Ownership

At any point in execution, each value has exactly one owner.

**Formalization**:
```
∀v, Ω. |{x | x: v^own ∈ Ω}| ≤ 1
```

### Invariant 2: No Use After Move

Moved values cannot be accessed.

**Formalization**:
```
Ω ⊢ e ok ⟹ FV(e) ∩ moved(Ω) = ∅
```

### Invariant 3: Borrow Exclusivity

Active mutable borrows are exclusive.

**Formalization**:
```
x: &'a mut τ ∈ Ω ⟹ ∄y. y: &'a τ ∈ Ω ∨ y: &'a mut τ ∈ Ω (y ≠ x)
```

## Ownership Soundness

### Theorem (Ownership Preservation)

If Ω ⊢ e : τ and e ⟶ e', then there exists Ω' such that Ω' ⊢ e' : τ
and Ω' respects the ownership invariants.

**Proof**: By induction on the typing derivation, showing that each reduction
step either:
1. Preserves ownership (no moves occur)
2. Correctly transfers ownership (move semantics)
3. Creates valid borrows (borrowing rules)
□

### Theorem (Move Safety)

A well-typed program never accesses a moved value.

**Proof**:
- The typing rules require x ∉ moved(Ω) for variable access
- Move operations mark variables as moved
- Subsequent access attempts will fail type checking
□

### Theorem (Double-Free Prevention)

A well-typed program never frees memory twice.

**Proof**:
- Each value has exactly one owner (Invariant 1)
- Only the owner can free memory (via dropping)
- After dropping, the owner is removed from Ω
- No other references to the freed memory exist (Invariant 2)
□

## Drop Semantics

### Drop Trait

```
trait Drop {
    fn drop(&mut self);
}
```

### Automatic Drop Insertion

At the end of each scope, insert drop calls for owned values:

```
fn insert_drops(scope, Ω):
    for x: τ^own ∈ Ω where τ: Drop:
        emit(drop(&mut x))
```

### Drop Order

Values are dropped in reverse declaration order:

```
{
    let a = ...;  // dropped last
    let b = ...;  // dropped second
    let c = ...;  // dropped first
}
```

## Examples

### Valid Move

```ml
let x = Box::new(42);
let y = x;            // x moved to y
println(y);           // OK
// println(x);        // ERROR: x has been moved
```

### Valid Borrow

```ml
let x = String::from("hello");
let y = &x;           // borrow x
println(y);           // OK
println(x);           // OK: x still valid
```

### Invalid Double Borrow

```ml
let mut x = 42;
let y = &mut x;       // mutable borrow
let z = &x;           // ERROR: cannot borrow while mutably borrowed
```

## Implementation

The ownership checker in `src/checker.rs` implements these rules through:

1. Tracking variable states (owned, borrowed, moved)
2. Validating borrow exclusivity
3. Checking move validity
4. Inserting drop calls

## TODO: Extensions

- [ ] Non-lexical lifetimes (NLL)
- [ ] Two-phase borrows
- [ ] Pin and self-referential structs
- [ ] Async ownership across await points
