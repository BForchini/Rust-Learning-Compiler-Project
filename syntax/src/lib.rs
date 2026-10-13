use thiserror::Error;

#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    Number(f64),
    Identifier(String),
    Fn,
    Equals,     // =
    Comma,      // ,
    Semicolon,  // ;
    Plus,       // +
    Minus,      // -
    Star,       // *
    Slash,      // /
    LeftParen,  // (
    RightParen, // )
}

#[derive(Debug, PartialEq, Error)]
pub enum CalculatorError {
    #[error("Failed to parse number: {0}")]
    NumberParse(#[from] std::num::ParseFloatError),
    #[error("Unexpected token while parsing expression")]
    ParseError,
    #[error("Input is Invalid")]
    InputInvalid,
    #[error("Division by zero occurred")]
    DivisionByZero,
    #[error("Ir generation error")]
    IrError,
    #[error("Undefined variable: {0}")]
    UndefinedVariable(String),
    #[error("Undefined function: {0}")]
    UndefinedFunction(String),
    #[error("Function {name} expects {expected} arguments, got {actual}")]
    FunctionArity {
        name: String,
        expected: usize,
        actual: usize,
    },
    #[error("Function defined more than once: {0}")]
    DuplicateFunction(String),
    #[error("Function name is reserved for the program entry point: {0}")]
    ReservedFunctionName(String),
}

#[derive(Debug, PartialEq, Clone)]
pub enum Expr {
    // For the parser to recognise order of operations.
    Number(f64),
    Variable(String),
    Assignment {
        name: String,
        value: Box<Expr>,
    },
    FunctionCall {
        name: String,
        arguments: Vec<Expr>,
    },
    Binary {
        left: Box<Expr>,
        operator: Operator,
        right: Box<Expr>,
    },
}

#[derive(Debug, PartialEq, Clone)]
pub enum Operator {
    Addition,
    Subtraction,
    Division,
    Multiplication,
}

#[derive(Debug, PartialEq, Clone)]
pub struct FunctionDefinition {
    pub name: String,
    pub parameters: Vec<String>,
    pub body: Expr,
}

#[derive(Debug, PartialEq, Clone)]
pub struct SourceProgram {
    pub functions: Vec<FunctionDefinition>,
    pub expression: Expr,
}

pub type Temp = usize;

#[derive(Debug, Clone, PartialEq)]
pub enum Instructions {
    LoadArgument {
        index: usize,
        destination: Temp,
    },
    LoadConstant {
        value: f64,
        destination: Temp,
    },
    Add {
        left: Temp,
        right: Temp,
        destination: Temp,
    },
    Subtract {
        left: Temp,
        right: Temp,
        destination: Temp,
    },
    Multiply {
        left: Temp,
        right: Temp,
        destination: Temp,
    },
    Divide {
        left: Temp,
        right: Temp,
        destination: Temp,
    },
    Call {
        name: String,
        arguments: Vec<Temp>,
        destination: Temp,
    },
}
