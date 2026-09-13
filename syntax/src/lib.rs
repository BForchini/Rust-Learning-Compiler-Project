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

/* 
#[derive(Debug, PartialEq, thiserror::Error)]
pub enum CompilerError {
    #[error("lexing failed: {0}")]
    Lex(#[from] LexerError),
    
    #[error("parsing failed: {0}")]
    Parse(#[from] ParseError),

    #[error("IR generation failed: {0}")]
    Ir(#[from] IrError),
}

let tokens = lex(input)?;
let ast = parse(tokens)?;
let ir = generate_ir(ast)?;
*/

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
