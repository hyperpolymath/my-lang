<!--
SPDX-License-Identifier: CC-BY-SA-4.0
Copyright (c) Jonathan D.A. Jewell <j.d.a.jewell@open.ac.uk>
-->
# SPDX-License-Identifier: CC-BY-SA-4.0
# Copyright (c) 2026 Jonathan D.A. Jewell (hyperpolymath) <j.d.a.jewell@open.ac.uk>

# My-Lang — System Specifications

My-Lang is a multi-dialect language with 4 dialects (Solo, Duet, Trio, Ensemble)
and an LLVM backend for compiled code generation.

## Memory Model

My-Lang has a split memory model between compiled and interpreted modes:

### Compiled Mode (LLVM)

- **Stack allocation**: Scalar values (integers, floats, booleans) are stack-allocated
  by default. LLVM's `alloca` instruction manages stack frames.
- **Heap allocation**: Compound values (strings, arrays, structs) are heap-allocated
  via `malloc`. Deallocation is explicit via `free` at scope exit.
- **No garbage collector**: The compiled mode does not include a GC. Memory is managed
  through deterministic scope-based deallocation and explicit free calls.
- **LLVM optimisations**: LLVM's optimisation passes (mem2reg, SROA) promote heap
  allocations to stack where possible, and eliminate unnecessary copies.
- **Value semantics**: Assignment copies values for scalars. Compound values use
  move semantics by default; explicit `copy` is required for duplication.
- **Alignment**: Memory layout follows the target platform's alignment rules as
  determined by LLVM's data layout string.

### Interpreted Mode

- **Reference counting**: Values in the interpreter use `Rc<T>` for memory
  management. Values are cloned when shared across scopes.
- **Environment maps**: Local variables are stored in HashMap-based environments
  with lexical scope chaining.
- **No cycle collection**: Cyclic references in interpreter mode are not collected.

## Concurrency Model

My-Lang provides several concurrency primitives across its dialects:

- **`go` blocks**: Goroutine-style lightweight concurrent tasks. `go` blocks spawn
  a new execution context that runs concurrently with the spawning code. In compiled
  mode, these map to OS threads with lightweight scheduling.
- **Channels**: Typed communication channels for passing values between `go` blocks.
  Channels are bounded by default with configurable buffer sizes.
- **`comptime` blocks**: Compile-time evaluation blocks that execute during
  compilation. These do not run concurrently at runtime but allow parallel
  computation during the build phase.
- **Dialect-specific concurrency**:
  - **Solo**: Single-threaded only. `go` blocks are syntactically available but
    execute sequentially.
  - **Duet**: Two concurrent contexts (producer/consumer pattern).
  - **Trio**: Three-way concurrency with mediator pattern.
  - **Ensemble**: Unrestricted concurrency with full `go` block and channel support.
- **Synchronisation**: `await` keyword blocks until a `go` block completes.
  `select` chooses among multiple channel operations.

## Effect System

My-Lang combines AI effects with standard async effects:

### AI Effects

- **`suggest`**: Requests an AI-generated suggestion for a value or implementation.
  The suggestion is non-binding and must be explicitly accepted.
- **`infer`**: Requests AI type inference or value inference based on context.
  Stronger than suggest — the inferred result is used unless overridden.
- **`generate`**: Requests AI code generation for a function body, test case, or
  data structure. Generated code is inserted at compile time.

### Standard Effects

- **`async`/`await`**: Asynchronous function execution. `async` marks a function
  as potentially suspending; `await` suspends until the result is available.
- **Error effects**: Functions that can fail return `Result` types. The `?` operator
  propagates errors up the call stack.
- **IO effects**: File, network, and console IO are tracked as effects in the type
  system. Pure functions cannot perform IO without explicit effect annotation.

### Effect Interaction

- AI effects are resolved at compile time and do not exist at runtime.
- Async effects compose with error effects (async functions can return Results).
- The dialect determines which effects are available (Solo restricts concurrency
  effects; Ensemble enables all).

## Module System

My-Lang uses a path-based import system:

- **`import` declarations**: Modules are imported by path relative to the project
  root: `import math.vector` resolves to `math/vector.ml`.
- **Selective imports**: Specific items can be imported:
  `import math.vector (Vec3, dot, cross)`.
- **`export` visibility**: Items are private by default. The `export` keyword makes
  functions, types, and constants visible to other modules.
- **Re-exports**: Modules can re-export imported items to create facade modules:
  `export import math.vector`.
- **Dialect namespacing**: Each dialect has its own standard library namespace.
  Ensemble modules can import from any dialect; Solo modules can only import from
  Solo and shared libraries.
- **Path resolution**: The compiler searches the project source tree, then the
  standard library, then any configured library paths.
- **No package manager yet**: External dependencies are vendored or referenced by
  path. A package manager is planned.
