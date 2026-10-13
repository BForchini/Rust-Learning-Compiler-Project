
# Rust Compiler Project

A programming language project I started to learn Rust and explore how compilers work.

It began as a simple calculator and has gradually developed into a small language implementation with separate lexer, parser and evaluator crates.

## Current State

The language currently supports:

* Floating-point arithmetic
* `+`, `-`, `*`, `/`
* Variable assignment and references
* Expression-bodied functions, parameters, and nested calls
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

Run `cargo run -p compiler` to exercise lexing, parsing, evaluation, IR generation, and ARM64 code generation from the same program. The compiler invokes `clang` to assemble and link `target/compiler-output/program.s` into `target/compiler-output/program`; `clang` must be installed. Native function calls currently support up to eight floating-point arguments.

The current pipeline is:

```text
Source → Lexer → Parser → AST → Evaluator / IR → ARM64 Backend
```

The project is structured as a Cargo workspace:

```text
my-language/
├── syntax/
├── lexer/
├── parser/
├── evaluator/
├── IR/
├── Arm64Backend/
└── compiler/
```

## What's Next

The goal is to gradually turn the project into a more complete compiler. Planned work includes:

* Control flow
* A custom intermediate representation
* Bytecode and a virtual machine
* Type checking
* Eventually, native code generation

The project is primarily a way for me to learn Rust and compiler design by building something from scratch rather than following a tutorial.
