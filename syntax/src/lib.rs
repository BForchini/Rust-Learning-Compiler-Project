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

#[derive(Debug, PartialEq)]
pub enum CalculatorError {
    // All possible errors that may occur
    InputInvalid,
    ParseError,
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
