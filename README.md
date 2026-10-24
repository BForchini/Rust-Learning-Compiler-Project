# Rust Compiler Project

A programming language project I started to learn Rust and explore how compilers work.

It began as a simple calculator and has gradually developed into a small language implementation with separate lexer, parser and evaluator crates.

## Current State

The language currently supports:

* Floating-point arithmetic
* `+`, `-`, `*`, `/`
* Variable assignment and references
* Expression-bodied functions, parameters, and nested calls
* `if` / `else` expressions and brace-delimited blocks
* Numeric console input with `input()` and output with `print(value)`
* Operator precedence
* Recursive-descent parsing
* Abstract syntax trees
* Basic error handling
* AST evaluation

For example, an assignment can initialize a variable with an arithmetic expression:

```text
x = 4 + 3
```

The assigned value can then be used in expressions. Operator precedence applies in initializers too, e.g. `x = 4 + 3 * 2`.

Functions are defined with `fn`, an optional comma-separated parameter list, and an expression body. Separate definitions with semicolons:

```text
fn add(a, b) = a + b; fn twice(x) = add(x, x); twice(4)
```

Run `cargo run -p compiler -- examples/calculator.calc` to run the calculator program written in this language. Follow its prompts and enter three numbers, each on its own line: the left operand, right operand, and operation code (`1` add, `2` subtract, `3` multiply, `4` divide). The program prints the result.

For example, piping `9`, `3`, and `4` into the program displays the prompts and prints `3`. An operation code outside `1`-`4` prints an error message instead of silently selecting an operation.

The current pipeline is:

```text
Source → Lexer → Parser → AST → Evaluator / IR → ARM64 Backend
```

What's Next

The goal is to gradually turn the project into a more complete compiler. Planned work includes:

* Conditional and loop expressions
* A custom intermediate representation
* Bytecode and a virtual machine
* Type checking
* Eventually, native code generation

The project is primarily a way for me to learn Rust and compiler design by building something from scratch rather than following a tutorial.
