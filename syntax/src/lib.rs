use thiserror::Error;

#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    Number(f64),
    String(String),
    Identifier(String),
    Fn,
    If,
    Else,
    For,
    While,
    LeftBrace,    // {
    RightBrace,   // }
    Equals,       // =
    EqualsEquals, // ==
    GreaterThan,  // >
    LessThan,     // <
    Comma,        // ,
    Semicolon,    // ;
    Plus,         // +
    Minus,        // -
    Star,         // *
    Slash,        // /
    LeftParen,    // (
    RightParen,   // )
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
    #[error("Function name is reserved: {0}")]
    ReservedFunctionName(String),
    #[error("Input/output error: {0}")]
    InputOutput(String),
    #[error("Input is not a valid number: {0}")]
    InvalidNumberInput(String),
}

#[derive(Debug, PartialEq, Clone)]
pub enum Expr {
    // For the parser to recognise order of operations.
    Number(f64),
    String(String),
    Variable(String),
    Assignment {
        name: String,
        value: Box<Expr>,
    },
    FunctionCall {
        name: String,
        arguments: Vec<Expr>,
    },
    Block(Vec<Expr>),
    If {
        condition: Box<Expr>,
        then_branch: Box<Expr>,
        else_branch: Box<Expr>,
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
    GreaterThan,
    LessThan,
    Equality,
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
    GreaterThan {
        left: Temp,
        right: Temp,
        destination: Temp,
    },
    LessThan {
        left: Temp,
        right: Temp,
        destination: Temp,
    },
    Equality {
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

#[cfg(test)]
mod tests {
    use super::{CalculatorError, Expr};

    #[test]
    fn block_and_if_expressions_are_cloneable_ast_nodes() {
        let expression = Expr::If {
            condition: Box::new(Expr::Number(1.0)),
            then_branch: Box::new(Expr::Block(vec![Expr::Number(2.0)])),
            else_branch: Box::new(Expr::Block(vec![Expr::Number(3.0)])),
        };

        assert_eq!(expression.clone(), expression);
    }

    #[test]
    fn input_and_output_errors_have_user_facing_messages() {
        assert_eq!(
            CalculatorError::InvalidNumberInput("nope".into()).to_string(),
            "Input is not a valid number: nope"
        );
        assert_eq!(
            CalculatorError::InputOutput("closed".into()).to_string(),
            "Input/output error: closed"
        );
    }
}
