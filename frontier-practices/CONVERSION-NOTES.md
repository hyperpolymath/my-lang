<!-- SPDX-License-Identifier: MPL-2.0 -->
# ReScript → AffineScript conversion notes

This documents the port of the Seven Tentacles agent system from ReScript
(`.res`) to AffineScript (`.affine`). Source of truth for syntax was the
reference repo cloned at `/tmp/affinescript-ref` (the OCaml compiler's
`lib/parser.mly`, the stdlib `.affine` modules, and the example/test
`.affine` files), **not** the tree-sitter grammar (which is a simplified
editor grammar and lags the real parser — e.g. its `use_decl` does not show
the `::{...}` / `as` forms the real parser accepts).

## Files

Converted (then the `.res` originals were deleted):

| ReScript | AffineScript |
|---|---|
| `agents/Types.res` | `agents/Types.affine` |
| `agents/RedAgent.res` | `agents/RedAgent.affine` |
| `agents/OrangeAgent.res` | `agents/OrangeAgent.affine` |
| `agents/YellowAgent.res` | `agents/YellowAgent.affine` |
| `agents/GreenAgent.res` | `agents/GreenAgent.affine` |
| `agents/BlueAgent.res` | `agents/BlueAgent.affine` |
| `agents/IndigoAgent.res` | `agents/IndigoAgent.affine` |
| `agents/VioletAgent.res` | `agents/VioletAgent.affine` |
| `tools/RevealSystem.res` | `tools/RevealSystem.affine` |

Build config: `rescript.json` removed; `Justfile` (`build-agents`, `watch`,
`status`), `deno.json` (imports + `build` task), and `docs/TENTACLES_MAP.adoc`
updated. Per-file compile commands documented in `BUILD.adoc`.

## ⚠️ Compile verification was NOT possible

No runnable AffineScript compiler was available in this environment:

* The compiler is a native OCaml binary built with `dune` — but `dune`,
  `ocaml`, and `opam` are **not installed** here (`bin/main.ml` + `dune`
  exist as source only; there is no `_build/`).
* `packages/affinescript-cli/mod.js` is only a *download shim*: it fetches a
  per-platform native binary from a GitHub Release and execs it. That needs
  network + a matching release artifact; neither is available offline.
* `deno` is not installed either (only `node` is present), so even the shim
  could not run.

**Therefore the `.affine` files have NOT been machine-checked.** They were
pattern-matched against the reference as carefully as possible. The
"verified" vs "inferred/guessed" breakdown is below. Run
`just build-agents` (or the commands in `BUILD.adoc`) once a compiler is
available and fix any residual errors.

## Mapping applied

| ReScript | AffineScript | Confidence |
|---|---|---|
| (file = module) | `module PascalName;` header | verified (`parser.mly` `module_decl`; stdlib uses it) |
| `open Types` | `use Types::{ A, B, ... };` (explicit names) | verified (`import_decl` ImportList; stdlib `use prelude::{...}`) |
| exported `let`/`type` | `pub` prefix | verified (`visibility: PUB`) |
| `type t = \| A \| B(p)` | `pub enum T { A, B(P) }` | verified (`enum_decl`; `comprehensive_test.affine`) |
| `type r = { f: T }` (+ mutually-rec `and`) | separate `pub struct R { f: T }` decls (no `and`) | verified (`struct_decl`); recursion handled by declaring structs before/independently of users |
| record literal `{ f: v }` | `#{ f: v }` | verified (`HASH_LBRACE`; `comprehensive_test.affine` `#{ x: 3, y: 4 }`) |
| `option<T>` / `Some` / `None` | `Option<T>` / `Some` / `None` (from `prelude`) | verified (`prelude.affine`) |
| `array<T>` / `[a,b]` / `arr[i]` | `[T]` / `[a, b]` / `arr[i]` | verified (`collections.affine`, `option.affine`) |
| `Array.length(a)` | `len(a)` builtin | verified (used throughout stdlib) |
| string/array concat `++` | `++` | verified (`prelude.affine`, `string.affine`) |
| `switch x { \| A => e }` | `match x { A => e, }` (comma arms, `_` wildcard) | verified (`option.affine`, `math.affine`) |
| tuple `(int,int)` / tuple patterns | `(Int, Int)` / `match (a,b) { (x,y) => ... }` | verified (`option.affine` `match (a, b) { (Some(x), ...) => ... }`) |
| `float` / Int→Float | `Float` / `float(n)` builtin | verified (`testing.affine` uses `float(iterations)`) |
| `bool` / `true` / `false` | `Bool` / `true` / `false` | verified |
| `Js.Math.random_int(0, n)` | `Deno::random_in_range(0, n)` | **inferred** — see RNG below |
| `arr[i]->Option.getOr(d)` | `random_element` helper (guard `len`, else fallback) | inferred — see array-index below |
| `module Red = RedAgent` | `use RedAgent as Red;` + qualified calls | verified the *syntax*; **the semantics forced a rename** — see collisions below |

## Non-trivial decisions

### 1. Top-level `let` → zero-arg `fn`

AffineScript's grammar (`top_level` in `parser.mly`) allows only `fn`,
`type`, `effect`, `trait`, `impl`, `const`, and `extern` at module top level —
**there is no top-level `let`**. `const` exists but every example in the
reference uses it for *scalars* only (`const PI: Float = ...`); no `.affine`
file uses a `const` initialised to a record/array/enum value.

So each ReScript top-level binding of a record/array value
(`let names`, `let personality`, `let teaches`, `let lessons`, `let agent`)
became a **zero-argument public function** returning that value:

```
pub fn names() -> AgentNames { #{ ... } }
```

and every reference was updated to a call: `names.cuttle` → `names().cuttle`,
`personality.encouragement` → `personality().encouragement`. This is the
idiom the whole stdlib uses (functions returning records/arrays) and is
definitely supported. Trade-off: the value is rebuilt on each call rather
than shared — semantically identical for this pure, immutable data.

### 2. RNG (`Js.Math.random_int`) — INFERRED

`stdlib/Deno.affine` declares `pub extern fn random_in_range(lo, hi) -> Int`
("uniform integer draw in `[lo, hi)`", lowered to
`Math.floor(Math.random()*(hi-lo))+lo` on the `--deno-esm` backend). That is
an exact match for ReScript `Js.Math.random_int(0, len)`, so `random_element`
in `Types.affine` uses it via `use Deno;` + `Deno::random_in_range(0, n)`.

I deliberately did **not** hand-roll `floor(math_random() * to_float(n))`
(the suggestion in the task) because:
* `floor`'s signature is `Float -> Int` (a builtin), and there is **no `*.`
  float-multiply operator** — `math.affine` multiplies floats with plain `*`
  (`degrees * PI / 180.0`). So the hand-rolled form would be
  `floor(math_random() * float(n))`, which works, but
* `random_in_range` is the purpose-built primitive and is clearer/safer.

**Caveat / coupling:** importing `Deno` ties `Types.affine` (hence every
agent) to a host providing `Deno::*` (the `--deno-esm` backend or the
`affine-deno` runtime shim). This is acceptable because the original targeted
JS/Deno (`Js.Math`), but a non-Deno backend would need the RNG call swapped.
**Unverified:** that `Deno::random_in_range` resolves cleanly for a
non-`main` library module under `check`.

### 3. Array-index "random element or fallback"

ReScript `arr[idx]->Option.getOr(fallback)` relies on indexing returning
`option`. In AffineScript `arr[i]` returns the **element directly** (verified:
`collections.affine` `list[0]`, `option.affine` `list[index]`), and `len`
gives the length. So the idiom became the `random_element(arr, fallback)`
helper in `Types.affine`:

```
pub fn random_element(arr: [String], fallback: String) -> String {
  let n = len(arr);
  if n > 0 { arr[Deno::random_in_range(0, n)] } else { fallback }
}
```

Behaviour preserved: a uniformly-random element, or the fallback when empty.
(The original could never actually hit the fallback for the non-empty
personality arrays; the helper keeps the same safety net.)

### 4. Module aliasing AND a forced rename — the important one

ReScript `module Red = RedAgent` + `Red.getName(stage)` namespaces each
agent. AffineScript supports `use RedAgent as Red;` and qualified value paths
`Red::red_get_name(x)` (verified in `parser.mly`: `upper_ident COLONCOLON
lower_ident` → `ExprField`, lowered by `Resolve.lower_qualified_value_paths`).

**However** — and this is the subtle part — `lower_qualified_value_paths`
**erases the qualifier**: `Red::red_get_name` is rewritten to the bare
`red_get_name`, and the module loader *flat-imports* every imported module's
public functions **by name into one namespace**, deduplicating on collision
(`module_loader.ml` `already_in`). If all seven agents each exported a
function literally named `get_name`, the seven `get_name`s would collide and
only the first would survive — the per-module qualifier cannot disambiguate
them because it is gone before resolution.

**Decision:** each agent's *stage-dispatched* public functions are uniquely
prefixed with the colour:
`red_get_name`, `orange_get_name`, …, `red_get_hidden_concept`, …,
`red_reveal_text`, …, `red_encourage`/`red_correct`/`red_celebrate`. The
data accessors (`names`, `personality`, `teaches`, `lessons`, `agent`) keep
their plain names because they are **only called within their own module**,
never cross-module, so they never collide in `RevealSystem`'s import set.

This changes the public API names of the per-agent functions but preserves
all behaviour and is the faithful way to compile. `RevealSystem` still reads
naturally: `Red::red_get_name(stage)`.

**Unverified:** whether the resolver is happy with `use X as Y;` where the
flattened public set is large; and whether `agent()`/`names()` etc. (same
name across modules) cause a problem *in RevealSystem* — they should not,
because `RevealSystem` never imports those names (it only imports
`Types::{...}`, `prelude::{...}`, and the seven agents *as aliases*; the
aliased ImportSimple still flat-imports the modules' public fns by name, so
the seven `names`/`personality`/`agent`/etc. **do** collide in
RevealSystem's flattened set). RevealSystem never *calls* those collided
names, so it is harmless for type-checking RevealSystem's own bodies — but if
the loader rejects duplicate flat imports outright (rather than deduping),
RevealSystem would fail to load. The observed loader behaviour
(`already_in` dedup, no error) suggests dedup, not rejection. **This is the
single most likely place to need a fix once a compiler is available** — the
robust fallback is to also prefix `names`/`personality`/`teaches`/`lessons`/
`agent` per colour (e.g. `red_agent()`), at the cost of more churn.

### 5. `filterMap` + `Option.map`

`RevealSystem.generateStageReveal` used
`colors->Array.filterMap(c => getRevealText(...)->Option.map(t => (c, t)))`.
The stdlib *does* provide `option::map_filter` and `option::map`, but they
are **not `pub`** (only `option::unwrap_or` is `pub`), so they cannot be
cross-module imported. Rather than depend on that, I inlined the logic with a
`for` loop + `match` accumulating into a `mut` array — the exact idiom
`option.affine`'s own `cat_options`/`map_filter` use internally:

```
let mut agentReveals = [];
for color in colors {
  match get_reveal_text(color, fromStage, toStage) {
    Some(text) => { agentReveals = agentReveals ++ [(color, text)]; },
    None => {}
  }
}
```

### 6. Float arithmetic & no string interpolation

* `Int.toFloat(a) /. Int.toFloat(b) *. 100.0` → `float(a) / float(b) * 100.0`.
  There is **no `*.`/`/.`** in AffineScript; `math.affine` uses plain `*`/`/`
  on `Float`. `float(n)` is the Int→Float builtin (used by `to_float`, which
  is itself not `pub`, so `float` is called directly).
* ReScript template literals `` `${x} ...` `` → `++` concatenation. No
  string-interpolation syntax exists in the reference (searched lexer/parser/
  spec). The multi-line `theBigReveal` string is an ordinary double-quoted
  literal containing newlines (the reference's string literals are not
  newline-restricted in any way I could find — **mildly unverified**; if the
  lexer rejects raw newlines in `"..."`, this needs `\n` escapes or
  line-wise `++`).

### 7. Naming conventions

* Types/enums/structs/variants → PascalCase (`stage`→`Stage`, `agentColor`→
  `AgentColor`, variants already PascalCase).
* Functions/fields → kept the original field names (records are structs with
  the same field identifiers) but multi-word top-level fns were snake_cased
  to match stdlib style (`stageToAge`→`stage_to_age`, `getRevealText`→
  `get_reveal_text`). Struct *field* names kept their original camelCase
  (`hiddenConcept`, `winCondition`, …) to preserve the data shape exactly.

## Things specifically left UNVERIFIED (no compiler to confirm)

1. Whether a library module with no `fn main()` type-checks under
   `affinescript check` (all stdlib modules are libraries, so this should be
   fine, but I could not run it).
2. The cross-module flat-import collision behaviour in `RevealSystem`
   (decision #4) — most likely fix-point if anything fails.
3. Raw newlines inside the `theBigReveal` string literal (#6).
4. That `use Deno;` resolves for a JS/Wasm target invoked via `check` without
   a backend flag (#2 RNG).
5. Trailing commas in `match` arms and multi-line `#{ }` literals — used
   pervasively in the reference, so high-confidence, but not run here.
6. Struct field *punning* was avoided entirely (always `field: value`), so no
   risk there even though ReScript used `{ names, teaches, ... }` shorthand.
