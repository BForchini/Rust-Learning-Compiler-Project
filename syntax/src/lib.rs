use thiserror::Error;

#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    Number(f64),
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
}

#[derive(Debug, PartialEq)]
pub enum Expr {
    // For the parser to recognise order of operations.
    Number(f64),
    Binary {
        left: Box<Expr>,
        operator: Operator,
        right: Box<Expr>,
    },
}

#[derive(Debug, PartialEq)]
pub enum Operator {
    Addition,
    Subtraction,
    Division,
    Multiplication,
}

pub type Temp = usize;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Instructions {
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
}
