# Grammar Reference (EBNF)

Complete formal grammar specification for My Language.

## Notation

| Symbol | Meaning |
|--------|---------|
| `::=` | Definition |
| `\|` | Alternative |
| `[ ]` | Optional (0 or 1) |
| `{ }` | Repetition (0 or more) |
| `( )` | Grouping |
| `" "` | Terminal string |
| `' '` | Terminal character |
| `/* */` | Comment |

## Lexical Grammar

### Whitespace and Comments

```ebnf
whitespace       ::= ' ' | '\t' | '\n' | '\r'
line_comment     ::= "//" { any_char - '\n' } '\n'
block_comment    ::= "/*" { any_char } "*/"
```

### Identifiers

```ebnf
identifier       ::= (letter | '_') { letter | digit | '_' }
letter           ::= 'a'..'z' | 'A'..'Z' | unicode_letter
digit            ::= '0'..'9'
```

### Literals

```ebnf
integer_literal  ::= decimal_literal | hex_literal | octal_literal | binary_literal
decimal_literal  ::= digit { digit | '_' }
hex_literal      ::= "0x" hex_digit { hex_digit | '_' }
octal_literal    ::= "0o" octal_digit { octal_digit | '_' }
binary_literal   ::= "0b" binary_digit { binary_digit | '_' }

hex_digit        ::= digit | 'a'..'f' | 'A'..'F'
octal_digit      ::= '0'..'7'
binary_digit     ::= '0' | '1'

float_literal    ::= digit { digit } '.' digit { digit } [ exponent ]
                   | digit { digit } exponent
exponent         ::= ('e' | 'E') ['+' | '-'] digit { digit }

string_literal   ::= '"' { string_char | escape_seq | interpolation } '"'
                   | 'r#"' { any_char } '"#'
                   | '"""' { any_char } '"""'
string_char      ::= any_char - ('"' | '\\' | '{')
escape_seq       ::= '\\' ('n' | 'r' | 't' | '\\' | '"' | '{' | '0' | 'x' hex_digit hex_digit)
interpolation    ::= '{' expression '}'

char_literal     ::= '\'' (char_char | escape_seq) '\''
char_char        ::= any_char - ('\'' | '\\')

bool_literal     ::= "true" | "false"
```

## Syntactic Grammar

### Program Structure

```ebnf
program          ::= { top_level_item }

top_level_item   ::= function_def
                   | struct_def
                   | enum_def
                   | trait_def
                   | impl_block
                   | type_alias
                   | const_def
                   | module_def
                   | use_decl
                   | ai_model_decl
                   | prompt_decl
                   | effect_def
```

### Functions

```ebnf
function_def     ::= { attribute } [ "pub" ] [ "async" ] "fn" identifier
                     [ generic_params ] "(" [ param_list ] ")" [ "->" type ]
                     [ where_clause ] [ contract_clause ] block

param_list       ::= param { "," param } [ "," ]
param            ::= [ "mut" ] identifier ":" type [ "=" expression ]

generic_params   ::= "<" generic_param { "," generic_param } ">"
generic_param    ::= identifier [ ":" type_bounds ]
                   | "const" identifier ":" type

type_bounds      ::= type { "+" type }

where_clause     ::= "where" where_predicate { "," where_predicate }
where_predicate  ::= type ":" type_bounds

contract_clause  ::= { requires_clause } { ensures_clause }
requires_clause  ::= "requires" expression
ensures_clause   ::= "ensures" expression
```

### Types

```ebnf
type             ::= path_type
                   | reference_type
                   | array_type
                   | tuple_type
                   | function_type
                   | ai_type
                   | effect_type

path_type        ::= path [ generic_args ]
path             ::= [ "::" ] identifier { "::" identifier }

reference_type   ::= "&" [ "mut" ] type
array_type       ::= "[" type [ ";" expression ] "]"
tuple_type       ::= "(" [ type { "," type } ] ")"
function_type    ::= "fn" "(" [ type_list ] ")" [ "->" type ] [ effect_annotation ]

generic_args     ::= "<" type { "," type } ">"

ai_type          ::= "AI" "<" type ">"
effect_type      ::= type "with" effect_list
effect_list      ::= identifier { "," identifier }
effect_annotation ::= "with" effect_list
```

### Structs and Enums

```ebnf
struct_def       ::= { attribute } [ "pub" ] "struct" identifier
                     [ generic_params ] struct_body [ invariant_clause ]

struct_body      ::= "{" [ struct_fields ] "}"
                   | "(" [ type_list ] ")" ";"
                   | ";"

struct_fields    ::= struct_field { "," struct_field } [ "," ]
struct_field     ::= { attribute } [ "pub" ] identifier ":" type

enum_def         ::= { attribute } [ "pub" ] "enum" identifier
                     [ generic_params ] "{" [ enum_variants ] "}"

enum_variants    ::= enum_variant { "," enum_variant } [ "," ]
enum_variant     ::= identifier [ enum_variant_data ]
enum_variant_data ::= "(" [ type_list ] ")"
                    | "{" [ struct_fields ] "}"

type_list        ::= type { "," type }

invariant_clause ::= "invariant" expression
```

### Traits and Implementations

```ebnf
trait_def        ::= { attribute } [ "pub" ] "trait" identifier
                     [ generic_params ] [ ":" type_bounds ] "{" { trait_item } "}"

trait_item       ::= trait_method | trait_type | trait_const

trait_method     ::= function_signature [ block ]
trait_type       ::= "type" identifier [ ":" type_bounds ] [ "=" type ] ";"
trait_const      ::= "const" identifier ":" type [ "=" expression ] ";"

impl_block       ::= { attribute } "impl" [ generic_params ] [ type "for" ] type
                     [ where_clause ] "{" { impl_item } "}"

impl_item        ::= function_def | type_alias | const_def
```

### Statements

```ebnf
statement        ::= let_stmt
                   | expression_stmt
                   | return_stmt
                   | break_stmt
                   | continue_stmt
                   | ai_stmt

let_stmt         ::= "let" [ "mut" ] pattern [ ":" type ] "=" expression ";"
expression_stmt  ::= expression ";"
return_stmt      ::= "return" [ expression ] ";"
break_stmt       ::= "break" [ expression ] ";"
continue_stmt    ::= "continue" ";"

ai_stmt          ::= "ai" ai_keyword "{" ai_body "}" ";"
```

### Expressions

```ebnf
expression       ::= assignment_expr

assignment_expr  ::= or_expr [ assignment_op assignment_expr ]
assignment_op    ::= "=" | "+=" | "-=" | "*=" | "/=" | "%="
                   | "&=" | "|=" | "^=" | "<<=" | ">>="

or_expr          ::= and_expr { "||" and_expr }
and_expr         ::= comparison_expr { "&&" comparison_expr }

comparison_expr  ::= bitor_expr { comparison_op bitor_expr }
comparison_op    ::= "==" | "!=" | "<" | ">" | "<=" | ">="

bitor_expr       ::= xor_expr { "|" xor_expr }
xor_expr         ::= bitand_expr { "^" bitand_expr }
bitand_expr      ::= shift_expr { "&" shift_expr }
shift_expr       ::= additive_expr { ("<<" | ">>") additive_expr }

additive_expr    ::= multiplicative_expr { ("+" | "-") multiplicative_expr }
multiplicative_expr ::= power_expr { ("*" | "/" | "%") power_expr }
power_expr       ::= unary_expr { "**" unary_expr }

unary_expr       ::= unary_op unary_expr | postfix_expr
unary_op         ::= "-" | "!" | "&" | "&mut" | "*"

postfix_expr     ::= primary_expr { postfix_op }
postfix_op       ::= call_expr | index_expr | field_access | method_call | try_op

call_expr        ::= "(" [ arg_list ] ")"
index_expr       ::= "[" expression "]"
field_access     ::= "." identifier
method_call      ::= "." identifier [ "::" generic_args ] "(" [ arg_list ] ")"
try_op           ::= "?"

arg_list         ::= arg { "," arg } [ "," ]
arg              ::= [ identifier ":" ] expression

primary_expr     ::= literal
                   | identifier
                   | path
                   | "self"
                   | tuple_expr
                   | array_expr
                   | struct_expr
                   | block_expr
                   | if_expr
                   | match_expr
                   | loop_expr
                   | while_expr
                   | for_expr
                   | closure_expr
                   | ai_expr
                   | "(" expression ")"

literal          ::= integer_literal
                   | float_literal
                   | string_literal
                   | char_literal
                   | bool_literal
```

### Control Flow

```ebnf
if_expr          ::= "if" expression block [ "else" ( if_expr | block ) ]

match_expr       ::= "match" expression "{" [ match_arms ] "}"
match_arms       ::= match_arm { "," match_arm } [ "," ]
match_arm        ::= pattern [ "if" expression ] "=>" ( expression | block )

loop_expr        ::= "loop" block
while_expr       ::= "while" expression block
for_expr         ::= "for" pattern "in" expression block

block            ::= "{" { statement } [ expression ] "}"
block_expr       ::= block
```

### Patterns

```ebnf
pattern          ::= or_pattern

or_pattern       ::= pattern_no_or { "|" pattern_no_or }

pattern_no_or    ::= literal_pattern
                   | identifier_pattern
                   | wildcard_pattern
                   | tuple_pattern
                   | struct_pattern
                   | enum_pattern
                   | slice_pattern
                   | range_pattern
                   | ref_pattern
                   | at_pattern

literal_pattern  ::= literal
identifier_pattern ::= [ "ref" ] [ "mut" ] identifier
wildcard_pattern ::= "_"
tuple_pattern    ::= "(" [ pattern { "," pattern } ] ")"
struct_pattern   ::= path "{" [ field_patterns ] "}"
field_patterns   ::= field_pattern { "," field_pattern } [ "," [ ".." ] ]
field_pattern    ::= identifier [ ":" pattern ]
enum_pattern     ::= path [ "(" [ pattern { "," pattern } ] ")" ]
                   | path [ "{" [ field_patterns ] "}" ]
slice_pattern    ::= "[" [ pattern { "," pattern } [ "," ".." [ identifier ] ] ] "]"
range_pattern    ::= [ literal ] ".." [ "=" ] [ literal ]
ref_pattern      ::= "&" [ "mut" ] pattern
at_pattern       ::= identifier "@" pattern
```

### Closures

```ebnf
closure_expr     ::= [ "move" ] closure_params [ "->" type ] ( expression | block )
closure_params   ::= "|" [ closure_param_list ] "|"
closure_param_list ::= closure_param { "," closure_param }
closure_param    ::= [ "mut" ] identifier [ ":" type ]
```

### Collections

```ebnf
tuple_expr       ::= "(" [ expression { "," expression } [ "," ] ] ")"
array_expr       ::= "[" [ expression { "," expression } [ "," ] ] "]"
                   | "[" expression ";" expression "]"
struct_expr      ::= path "{" [ field_inits ] "}"
field_inits      ::= field_init { "," field_init } [ "," ] [ ".." expression ]
field_init       ::= identifier [ ":" expression ]
```

### AI Constructs

```ebnf
ai_model_decl    ::= { attribute } "ai_model" identifier "{" { ai_config_field } "}"
ai_config_field  ::= identifier ":" expression

prompt_decl      ::= { attribute } "prompt" identifier [ generic_params ]
                     "(" [ param_list ] ")" [ "->" type ] prompt_body
prompt_body      ::= "{" string_literal "}"
                   | block

ai_expr          ::= "ai" "!" "{" expression "}"
                   | "ai" ai_keyword ai_body
                   | "ai" ai_keyword "(" [ arg_list ] ")"
                   | identifier "!" "(" [ arg_list ] ")"

ai_keyword       ::= "query" | "verify" | "generate" | "embed"
                   | "classify" | "optimize" | "test" | "infer"
                   | "constrain" | "validate" | "stream"

ai_body          ::= "{" { ai_field } "}"
ai_field         ::= identifier ":" expression

ai_stmt          ::= "ai" ai_keyword "{" ai_body "}" ";"
```

### Effects

```ebnf
effect_def       ::= { attribute } "effect" identifier [ generic_params ]
                     "{" { effect_operation } "}"

effect_operation ::= "fn" identifier "(" [ param_list ] ")" [ "->" type ] ";"

handle_expr      ::= "handle" expression "{" { handler_clause } "}"
handler_clause   ::= identifier "(" [ pattern_list ] ")" "=>" expression ","
```

### Modules and Imports

```ebnf
module_def       ::= { attribute } [ "pub" ] "mod" identifier ( ";" | "{" { top_level_item } "}" )

use_decl         ::= { attribute } [ "pub" ] "use" use_tree ";"
use_tree         ::= path [ "::" ( "*" | "{" use_trees "}" | "as" identifier ) ]
use_trees        ::= use_tree { "," use_tree } [ "," ]
```

### Attributes

```ebnf
attribute        ::= "#" "[" attr_content "]"
                   | "#" "!" "[" attr_content "]"

attr_content     ::= identifier [ attr_args ]
attr_args        ::= "(" attr_arg_list ")"
                   | "=" literal

attr_arg_list    ::= attr_arg { "," attr_arg }
attr_arg         ::= identifier [ "=" expression ]
                   | expression
```

### Type Aliases and Constants

```ebnf
type_alias       ::= { attribute } [ "pub" ] "type" identifier [ generic_params ]
                     "=" type ";"

const_def        ::= { attribute } [ "pub" ] "const" identifier ":" type "=" expression ";"
```

### Async and Concurrency

```ebnf
async_block      ::= "async" block
await_expr       ::= expression "." "await"
spawn_expr       ::= "spawn" block
go_expr          ::= "go" block
yield_stmt       ::= "yield" [ expression ] ";"
```

## Grammar Notes

### Ambiguity Resolution

1. **Generics vs Comparison**: `a<b>c` is parsed as `a::<b>(c)` if `a` is a function, otherwise as `(a < b) > c`

2. **Struct vs Block**: `{ field: value }` after an identifier is a struct literal, otherwise a block

3. **Closure vs Or-Pattern**: `|` is closure if at expression position, or-pattern if in pattern position

### Semicolon Insertion

Semicolons are required after:
- Let statements
- Expression statements (except block expressions)
- Return/break/continue statements

Semicolons are optional after:
- Block expressions at statement position
- If/match/loop expressions at statement position

### Reserved for Future

The following are reserved but not currently used:
- `abstract`, `become`, `box`, `do`, `final`
- `macro`, `override`, `priv`, `typeof`
- `unsized`, `virtual`, `yield`
