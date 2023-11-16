## Quickstart

```
> cargo build
> cargo run -- --input-file examples/simple_add.calc

```


## Language Spec

The programs are simple. See the `examples` dir for examples. Calc programs are of the form

```
let <ident_1> = <expr_1>;
let <ident_2> = <expr_2>;
...
let <ident_n> = <expr_n>;
<expr>
```

where

- `<ident_i>` is an identifier defined by the pest rule `{ ASCII_ALPHA ~ (ASCII_ALPHANUMERIC | "_" * }`

- `<expr_i>` is any arithmetic expression* over numbers of type `i32` and variables `{<ident_j> | j < i }` (i.e. no referencing of yet-undeclared variables is allowed)

- `<expr>` is any arithmetic expression* over numbers of type `i32` and any declared variables.


`*` arithmetic expressions can use parentheses, unary negation, and binary operators `+,-,*` and `^` (exponentiation)
