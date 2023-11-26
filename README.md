## Quickstart

Run with an initial context (required)
```
> cargo run -- --input-file examples/poly.calc --context examples/poly.json
```

Inspect the AST in json format:
```
> cargo run -- --input-file examples/simple_add.calc --serialize
```

## Language Spec

The programs are simple. See the `examples` dir for examples. Calc programs are of the form

```
pub <var_1>;
...
pub <var_m>;
let <ident_1> = <expr_1>;
let <ident_2> = <expr_2>;
...
let <ident_n> = <expr_n>;
<expr>
```

where

- `pub <var_i>;` is a declaration of a public variable, which represents a value that must be supplied to run the program. To provide the value for the interpreter, pass it in a json file via then `--context` arg (see examples).

- `<ident_i>` is an identifier defined by the pest rule `{ ASCII_ALPHA ~ (ASCII_ALPHANUMERIC | "_" * }`

- `<expr_i>` is any arithmetic expression* over Fields of type `i32`, declared public variables, and any other bound identifier. You cannot have identifiers which are mutually recursive, but the order does not matter.

- `<expr>` is any arithmetic expression* over Fields of type `i32`, any public variables, and any declared variables  .


`*` arithmetic expressions can use parentheses, unary negation, and binary operators `+,-,*` and `^` (exponentiation)
