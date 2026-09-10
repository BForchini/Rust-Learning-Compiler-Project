
# Rust Compiler Project

A programming language project I started to learn Rust and explore how compilers work.

It began as a simple calculator and has gradually developed into a small language implementation with separate lexer, parser and evaluator crates.

## Current State

The language currently supports:

* Floating-point arithmetic
* `+`, `-`, `*`, `/`
* Operator precedence
* Recursive-descent parsing
* Abstract syntax trees
* Basic error handling
* AST evaluation

For example:

```text
3 + 5 * 2
```

is parsed and evaluated to:

```text
13
```

The current pipeline is:

```text
Source → Lexer → Parser → AST → Evaluator
```

The project is structured as a Cargo workspace:

```text
my-language/
├── syntax/
├── lexer/
├── parser/
├── evaluator/
└── compiler/
```

## What's Next

The goal is to gradually turn the project into a more complete compiler. Planned work includes:

* Parentheses and variables
* Functions and control flow
* A custom intermediate representation
* Bytecode and a virtual machine
* Type checking
* Eventually, native code generation

The project is primarily a way for me to learn Rust and compiler design by building something from scratch rather than following a tutorial.
