#[derive(Debug, PartialEq)]
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
    InputInvalid,
    ParseError,
}
