
# Rust Learning Compiler Project

A small programming language and compiler project written in Rust.

The project started as a simple calculator and is being developed incrementally into a more complete language implementation. The main purpose is to learn how programming languages work while improving my Rust and systems programming skills.

## Current State

The language currently supports basic arithmetic expressions involving floating-point numbers:

* Addition
* Subtraction
* Multiplication
* Division
* Operator precedence
* Left-associative operations
* Basic lexical and parsing errors
* Evaluation of the resulting AST

For example:

```text
3 + 5 * 2
```

is evaluated as:

```text
13
```

The current execution path is:

```text
Source Code
    Lexer
    Parser
    AST
    Evaluator
    Result
```

The evaluator currently walks the AST directly rather than generating machine code.

## Project Structure

The project is organised as a Cargo workspace:

```text
my-language/
├── Cargo.toml
├── syntax/
│   ├── Cargo.toml
│   └── src/
├── lexer/
│   ├── Cargo.toml
│   └── src/
├── parser/
│   ├── Cargo.toml
│   └── src/
├── evaluator/
│   ├── Cargo.toml
│   └── src/
└── compiler/
    ├── Cargo.toml
    └── src/
```

Each crate has a separate responsibility.

### `syntax`

Contains the data structures shared between the different stages of the language implementation.

This currently includes:

* `Token`
* `Expr`
* `Operator`
* `CalculatorError`

Keeping these types in a separate crate means that the lexer, parser and evaluator can depend on the same definitions without depending directly on one another.

### `lexer`

Converts source code into a sequence of tokens.

For example:

```text
3 + 5 * 2
```

becomes approximately:

```text
Number(3)
Plus
Number(5)
Star
Number(2)
```

The lexer is responsible for recognising the individual pieces of the language, but not for determining how they fit together.

### `parser`

Converts the token stream into an abstract syntax tree.

The parser uses recursive descent and currently follows this grammar:

```text
expression → term (("+" | "-") term)*
term       → factor (("*" | "/") factor)*
factor     → number
```

This gives multiplication and division higher precedence than addition and subtraction.

For example:

```text
3 + 5 * 2
```

produces an AST conceptually equivalent to:

```text
    +
   / \
  3   *
     / \
    5   2
```

### `evaluator`

Evaluates the AST recursively and produces an `f64` result.

For a binary expression, the evaluator evaluates the left and right subexpressions before applying the operator.

This provides a simple execution mechanism while the rest of the language is being developed.

### `compiler`

Contains the executable entry point and will eventually coordinate the complete compilation process.

At the moment, the project is primarily an interpreter rather than a native-code compiler. The compiler crate will become more important once an intermediate representation and code generation are introduced.

## Architecture

The current architecture is deliberately simple:

```text
Source
  |
Lexer
  |
Tokens
  |
Parser
  |
AST
  |
Evaluator
  |
Value
```

The eventual goal is to introduce an intermediate representation between the AST and execution:

```text
Source
  |
Lexer
  |
Parser
  |
AST
  |
IR
  |
Bytecode / VM
```

Eventually, the project may also support native code generation.

## Example

Given:

```text
3 + 5 * 2
```

the lexer produces tokens representing the numbers and operators.

The parser then builds an AST which represents:

```text
3 + (5 * 2)
```

The evaluator recursively processes this tree:

```text
5 * 2 = 10
3 + 10 = 13
```

and returns:

```text
13
```

## Development Plan

The project is being developed by building a working version of each stage before expanding it further. The intention is not to completely finish one component before moving on to the next.

### Language features

The next language features I intend to experiment with include:

* Parentheses
* Variables
* Assignment
* Comparisons
* Boolean expressions
* `if` expressions/statements
* `while` loops
* Functions
* Scopes
* A basic type system

The exact order may change as the project develops.

### Intermediate representation

Once the front end is sufficiently developed, an intermediate representation will be added between the AST and execution.

The first version will likely be fairly simple. For example:

```text
LOAD 3
LOAD 5
LOAD 2
MUL
ADD
```

This could represent:

```text
3 + 5 * 2
```

A stack-based representation like this provides a relatively simple way to learn about bytecode and virtual machines before moving towards more complicated compiler architectures.

### Virtual machine

After introducing an IR, the next step will be a small virtual machine capable of executing it.

This should introduce concepts such as:

* Stack-based execution
* Instructions
* Registers or virtual registers
* Program counters
* Memory
* Control flow
* Function calls

### More advanced compiler infrastructure

As the language becomes more complicated, the project can move towards a more conventional compiler architecture involving:

* Basic blocks
* Control-flow graphs
* Loads and stores
* Branch instructions
* Function calls and returns
* Type checking
* Optimisation
* More structured IR

### Code generation

The eventual goal is to experiment with generating native code.

Possible approaches include using LLVM or implementing a small code-generation backend directly.

## What I am Learning

The project is primarily an opportunity to learn Rust through a reasonably substantial systems-oriented project.

Topics covered or planned include:

* Rust ownership and borrowing
* Error handling
* Traits and generics
* Modules and crates
* Testing
* Lexical analysis
* Recursive-descent parsing
* Abstract syntax trees
* Interpreters
* Intermediate representations
* Bytecode
* Virtual machines
* Control-flow graphs
* Type systems
* Optimisation
* Code generation

## Development Philosophy

The project is intentionally being built in stages.

Rather than trying to design a complete programming language from the beginning, each stage should produce something that works. New language features can then be added by extending the lexer, parser, AST and execution system as necessary.

This also means that parts of the implementation will probably be rewritten as the language becomes more sophisticated. That is intentional: understanding why an early design no longer works is part of the learning process.

## Long-Term Goal

The long-term goal is to turn this from a small arithmetic language into a functioning compiled language with its own frontend, intermediate representation and execution/code-generation backend.

The project is also intended to serve as a practical way of developing my Rust and computer science knowledge alongside my mathematics degree.
