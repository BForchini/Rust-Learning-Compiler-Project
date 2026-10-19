use syntax::CalculatorError::ParseError;
use syntax::{CalculatorError, Expr, Operator, Token};

#[derive(Debug, PartialEq)]
pub struct Parser {
    tokens: Vec<Token>,
    position: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser {
            tokens,
            position: 0,
        }
    }

    fn parse_factor(&mut self) -> Result<Expr, CalculatorError> {
        match self.peek().cloned() {
            Some(Token::Number(value)) => {
                self.position += 1;
                Ok(Expr::Number(value))
            }
            Some(Token::String(value)) => {
                self.position += 1;
                Ok(Expr::String(value))
            }
            Some(Token::Identifier(name)) => {
                self.position += 1;
                if self.peek() == Some(&Token::LeftParen) {
                    self.position += 1;
                    let mut arguments = Vec::new();
                    if self.peek() != Some(&Token::RightParen) {
                        loop {
                            arguments.push(self.parse_expression()?);
                            if self.peek() == Some(&Token::Comma) {
                                self.position += 1;
                            } else {
                                break;
                            }
                        }
                    }
                    if self.peek() != Some(&Token::RightParen) {
                        return Err(ParseError);
                    }
                    self.position += 1;
                    Ok(Expr::FunctionCall { name, arguments })
                } else {
                    Ok(Expr::Variable(name))
                }
            }
            Some(Token::LeftParen) => {
                self.position += 1;
                let inner = self.parse_expression()?;
                match self.peek() {
                    Some(Token::RightParen) => {
                        self.position += 1;
                        Ok(inner)
                    }
                    _ => Err(ParseError),
                }
            }
            _ => Err(ParseError),
        }
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.position)
    }

    fn parse_brack(&mut self) -> Result<Expr, CalculatorError> {
        self.parse_factor()
    }

    fn parse_term(&mut self) -> Result<Expr, CalculatorError> {
        // This is for multiplication and division
        let mut left = self.parse_brack()?;
        loop {
            match self.peek() {
                Some(Token::Star) => {
                    self.position += 1;
                    let right = self.parse_brack()?;
                    left = Expr::Binary {
                        left: Box::new(left),
                        operator: Operator::Multiplication,
                        right: Box::new(right),
                    };
                }
                Some(Token::Slash) => {
                    self.position += 1;
                    let right = self.parse_brack()?;
                    left = Expr::Binary {
                        left: Box::new(left),
                        operator: Operator::Division,
                        right: Box::new(right),
                    };
                }
                _ => break,
            }
        }
        Ok(left)
    }

    pub fn parse_expression(&mut self) -> Result<Expr, CalculatorError> {
        if let (Some(Token::Identifier(name)), Some(Token::Equals)) = (
            self.peek(),
            self.tokens.get(self.position.saturating_add(1)),
        ) {
            let name = name.clone();
            self.position += 2;
            return Ok(Expr::Assignment {
                name,
                value: Box::new(self.parse_expression()?),
            });
        }

        // This is for addition and subtraction
        let mut left = self.parse_term()?;
        loop {
            match self.peek() {
                Some(Token::Plus) => {
                    self.position += 1;
                    let right = self.parse_term()?;
                    left = Expr::Binary {
                        left: Box::new(left),
                        operator: Operator::Addition,
                        right: Box::new(right),
                    };
                }
                Some(Token::Minus) => {
                    self.position += 1;
                    let right = self.parse_term()?;
                    left = Expr::Binary {
                        left: Box::new(left),
                        operator: Operator::Subtraction,
                        right: Box::new(right),
                    };
                }
                Some(Token::GreaterThan) => {
                    self.position += 1;
                    let right = self.parse_term()?;
                    left = Expr::Binary {
                        left: Box::new(left),
                        operator: Operator::GreaterThan,
                        right: Box::new(right),
                    };
                }
                Some(Token::LessThan) => {
                    self.position += 1;
                    let right = self.parse_term()?;
                    left = Expr::Binary {
                        left: Box::new(left),
                        operator: Operator::LessThan,
                        right: Box::new(right),
                    };
                }
                Some(Token::EqualsEquals) => {
                    self.position += 1;
                    let right = self.parse_term()?;
                    left = Expr::Binary {
                        left: Box::new(left),
                        operator: Operator::Equality,
                        right: Box::new(right),
                    };
                }
                _ => break,
            }
        }
        Ok(left)
    }

    pub fn parse(&mut self) -> Result<Expr, CalculatorError> {
        let expression = self.parse_expression()?;
        if self.peek().is_some() {
            return Err(ParseError);
        }
        Ok(expression)
    }

    fn parse_function_definition(&mut self) -> Result<syntax::FunctionDefinition, CalculatorError> {
        if self.peek() != Some(&Token::Fn) {
            return Err(ParseError);
        }
        self.position += 1;
        let Some(Token::Identifier(name)) = self.peek().cloned() else {
            return Err(ParseError);
        };
        self.position += 1;
        if self.peek() != Some(&Token::LeftParen) {
            return Err(ParseError);
        }
        self.position += 1;

        let mut parameters = Vec::new();
        if self.peek() != Some(&Token::RightParen) {
            loop {
                let Some(Token::Identifier(parameter)) = self.peek().cloned() else {
                    return Err(ParseError);
                };
                if parameters.contains(&parameter) {
                    return Err(ParseError);
                }
                parameters.push(parameter);
                self.position += 1;
                if self.peek() == Some(&Token::Comma) {
                    self.position += 1;
                } else {
                    break;
                }
            }
        }
        if self.peek() != Some(&Token::RightParen) {
            return Err(ParseError);
        }
        self.position += 1;
        if self.peek() != Some(&Token::Equals) {
            return Err(ParseError);
        }
        self.position += 1;
        let body = self.parse_expression()?;
        if self.peek() != Some(&Token::Semicolon) {
            return Err(ParseError);
        }
        self.position += 1;

        Ok(syntax::FunctionDefinition {
            name,
            parameters,
            body,
        })
    }

    pub fn parse_program(&mut self) -> Result<syntax::SourceProgram, CalculatorError> {
        let mut functions = Vec::new();
        while self.peek() == Some(&Token::Fn) {
            let function = self.parse_function_definition()?;
            if function.name == "main" {
                return Err(CalculatorError::ReservedFunctionName(function.name));
            }
            if functions
                .iter()
                .any(|defined: &syntax::FunctionDefinition| defined.name == function.name)
            {
                return Err(CalculatorError::DuplicateFunction(function.name));
            }
            functions.push(function);
        }

        let expression = self.parse_expression()?;
        if self.peek().is_some() {
            return Err(ParseError);
        }
        Ok(syntax::SourceProgram {
            functions,
            expression,
        })
    }
}

#[cfg(test)]
pub mod tests {

    use std::vec;

    use syntax::{
        CalculatorError::ParseError,
        Expr::{self, Number},
        FunctionDefinition, Operator, SourceProgram, Token,
    };

    use crate::Parser;

    #[test]
    fn only_number() {
        let tokens = vec![Token::Number(3.0)];

        let mut parser = Parser::new(tokens);

        let result = parser.parse_factor();

        assert_eq!(result, Ok(Expr::Number(3.0)));
    }

    #[test]
    fn assignment_expression() {
        let mut parser = Parser::new(vec![
            Token::Identifier("x".into()),
            Token::Equals,
            Token::Number(4.0),
            Token::Plus,
            Token::Number(3.0),
        ]);

        assert_eq!(
            parser.parse(),
            Ok(Expr::Assignment {
                name: "x".into(),
                value: Box::new(Expr::Binary {
                    left: Box::new(Number(4.0)),
                    operator: Operator::Addition,
                    right: Box::new(Number(3.0)),
                }),
            })
        );
    }

    #[test]
    fn comparison_expression() {
        let mut parser = Parser::new(vec![
            Token::Identifier("x".into()),
            Token::LessThan,
            Token::Number(5.0),
            Token::EqualsEquals,
            Token::Number(3.0),
        ]);

        assert_eq!(
            parser.parse(),
            Ok(Expr::Binary {
                left: Box::new(Expr::Binary {
                    left: Box::new(Expr::Variable("x".into())),
                    operator: Operator::LessThan,
                    right: Box::new(Number(5.0)),
                }),
                operator: Operator::Equality,
                right: Box::new(Number(3.0)),
            })
        );
    }

    #[test]
    fn parses_functions_and_nested_calls() {
        let tokens = vec![
            Token::Fn,
            Token::Identifier("add".into()),
            Token::LeftParen,
            Token::Identifier("a".into()),
            Token::Comma,
            Token::Identifier("b".into()),
            Token::RightParen,
            Token::Equals,
            Token::Identifier("a".into()),
            Token::Plus,
            Token::Identifier("b".into()),
            Token::Semicolon,
            Token::Fn,
            Token::Identifier("twice".into()),
            Token::LeftParen,
            Token::Identifier("x".into()),
            Token::RightParen,
            Token::Equals,
            Token::Identifier("add".into()),
            Token::LeftParen,
            Token::Identifier("x".into()),
            Token::Comma,
            Token::Identifier("x".into()),
            Token::RightParen,
            Token::Semicolon,
            Token::Identifier("twice".into()),
            Token::LeftParen,
            Token::Number(4.0),
            Token::RightParen,
        ];
        let mut parser = Parser::new(tokens);

        assert_eq!(
            parser.parse_program(),
            Ok(SourceProgram {
                functions: vec![
                    FunctionDefinition {
                        name: "add".into(),
                        parameters: vec!["a".into(), "b".into()],
                        body: Expr::Binary {
                            left: Box::new(Expr::Variable("a".into())),
                            operator: Operator::Addition,
                            right: Box::new(Expr::Variable("b".into())),
                        },
                    },
                    FunctionDefinition {
                        name: "twice".into(),
                        parameters: vec!["x".into()],
                        body: Expr::FunctionCall {
                            name: "add".into(),
                            arguments: vec![Expr::Variable("x".into()), Expr::Variable("x".into()),],
                        },
                    },
                ],
                expression: Expr::FunctionCall {
                    name: "twice".into(),
                    arguments: vec![Number(4.0)],
                },
            })
        );
    }

    #[test]
    fn parse_error() {
        let tokens = vec![Token::Number(3.0), Token::Slash];

        let mut parser = Parser::new(tokens);

        let result = parser.parse_expression();

        assert_eq!(result, Err(ParseError));
    }

    #[test]
    fn with_star() {
        let tokens = vec![Token::Number(3.0), Token::Star, Token::Number(5.0)];

        let mut parser = Parser::new(tokens);

        let result = parser.parse_term();

        assert_eq!(
            result,
            Ok(Expr::Binary {
                left: Box::new(Expr::Number(3.0)),
                operator: Operator::Multiplication,
                right: Box::new(Expr::Number(5.0))
            })
        );
    }

    #[test]
    fn multi_number() {
        let tokens = vec![
            Token::Number(3.0),
            Token::Star,
            Token::Number(5.0),
            Token::Star,
            Token::Number(4.0),
        ];

        let mut parser = Parser::new(tokens);

        let result = parser.parse_term();

        assert_eq!(
            result,
            Ok(Expr::Binary {
                left: Box::new(Expr::Binary {
                    left: Box::new(Expr::Number(3.0)),
                    operator: Operator::Multiplication,
                    right: Box::new(Expr::Number(5.0))
                }),
                operator: Operator::Multiplication,
                right: Box::new(Expr::Number(4.0))
            })
        );
    }

    #[test]
    fn brackets() {
        let tokens = vec![Token::LeftParen, Token::Number(3.0), Token::RightParen];

        let mut parser = Parser::new(tokens);

        let result = parser.parse_expression();

        assert_eq!(result, Ok(Expr::Number(3.0)));
    }

    #[test]
    fn division() {
        let tokens = vec![Token::Number(3.0), Token::Slash, Token::Number(5.0)];

        let mut parser = Parser::new(tokens);

        let result = parser.parse_term();

        assert_eq!(
            result,
            Ok(Expr::Binary {
                left: Box::new(Number(3.0)),
                operator: Operator::Division,
                right: Box::new(Number(5.0))
            })
        );
    }

    #[test]
    fn multi_operator() {
        let tokens = vec![
            Token::Number(3.0),
            Token::Slash,
            Token::Number(5.0),
            Token::Star,
            Token::Number(4.0),
        ];

        let mut parser = Parser::new(tokens);

        let result = parser.parse_term();

        assert_eq!(
            result,
            Ok(Expr::Binary {
                left: Box::new(Expr::Binary {
                    left: Box::new(Expr::Number(3.0)),
                    operator: Operator::Division,
                    right: Box::new(Expr::Number(5.0))
                }),
                operator: Operator::Multiplication,
                right: Box::new(Expr::Number(4.0))
            })
        );
    }

    #[test]
    fn addition() {
        let tokens = vec![Token::Number(3.0), Token::Plus, Token::Number(5.0)];

        let mut parser = Parser::new(tokens);

        let result = parser.parse_expression();

        assert_eq!(
            result,
            Ok(Expr::Binary {
                left: Box::new(Number(3.0)),
                operator: Operator::Addition,
                right: Box::new(Number(5.0))
            })
        );
    }

    #[test]
    fn addition_and_multiplication() {
        let tokens = vec![
            Token::Number(3.0),
            Token::Star,
            Token::Number(5.0),
            Token::Plus,
            Token::Number(4.0),
        ];

        let mut parser = Parser::new(tokens);

        let result = parser.parse_expression();

        assert_eq!(
            result,
            Ok(Expr::Binary {
                left: Box::new(Expr::Binary {
                    left: Box::new(Expr::Number(3.0)),
                    operator: Operator::Multiplication,
                    right: Box::new(Expr::Number(5.0))
                }),
                operator: Operator::Addition,
                right: Box::new(Expr::Number(4.0))
            })
        );
    }

    #[test]
    fn addition_and_addition() {
        let tokens = vec![
            Token::Number(3.0),
            Token::Plus,
            Token::Number(5.0),
            Token::Plus,
            Token::Number(4.0),
        ];

        let mut parser = Parser::new(tokens);

        let result = parser.parse_expression();

        assert_eq!(
            result,
            Ok(Expr::Binary {
                left: Box::new(Expr::Binary {
                    left: Box::new(Expr::Number(3.0)),
                    operator: Operator::Addition,
                    right: Box::new(Expr::Number(5.0))
                }),
                operator: Operator::Addition,
                right: Box::new(Expr::Number(4.0))
            })
        );
    }

    #[test]
    fn addition_and_multiplication_reversed() {
        let tokens = vec![
            Token::Number(3.0),
            Token::Plus,
            Token::Number(5.0),
            Token::Star,
            Token::Number(4.0),
        ];

        let mut parser = Parser::new(tokens);

        let result = parser.parse_expression();

        assert_eq!(
            result,
            Ok(Expr::Binary {
                left: Box::new(Number(3.0)),
                operator: Operator::Addition,
                right: Box::new(Expr::Binary {
                    left: Box::new(Expr::Number(5.0)),
                    operator: Operator::Multiplication,
                    right: Box::new(Expr::Number(4.0))
                })
            })
        );
    }

    #[test]
    fn brackets_multi_number_test() {
        let tokens = vec![
            Token::LeftParen,
            Token::Number(3.0),
            Token::Plus,
            Token::Number(5.0),
            Token::RightParen,
            Token::Star,
            Token::Number(4.0),
        ]; // (3.0 + 5.0) * 4.0

        let mut parser = Parser::new(tokens);

        let result = parser.parse_expression();

        assert_eq!(
            result,
            Ok(Expr::Binary {
                left: Box::new(Expr::Binary {
                    left: Box::new(Expr::Number(3.0)),
                    operator: Operator::Addition,
                    right: Box::new(Expr::Number(5.0))
                }),
                operator: Operator::Multiplication,
                right: Box::new(Expr::Number(4.0))
            })
        );
    }
}
