use syntax::CalculatorError::{InputInvalid, ParseError};
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
        
        /*
        if self.position > self.tokens.len() {
            todo!();
        }
        */
        match &self.tokens[self.position] {
            Token::Number(value) => {
                self.position += 1;
                Ok(Expr::Number(*value))
            }
            _ => {
                Err(ParseError)
            }
        }
    }

    fn parse_term(&mut self) -> Result<Expr, CalculatorError> {
        let left = self.parse_factor()?;
        match &self.tokens[self.position] {
            Token::Star => {
                self.position += 1;
                Ok(Operator::Multiplication)
            }
            _ => Err(ParseError),
        }
    }
}

fn parse_expression() {
    todo!()
}


#[cfg(test)]
pub mod tests {

    use syntax::{Expr::{self, Number}, Token::{self, Star}};

    use crate::Parser;

    #[test]
    fn only_number() {
        let tokens = vec![Token::Number(3.0)];

        let mut parser = Parser::new(tokens);

        let result = parser.parse_factor();

        assert_eq!(result, Ok(Expr::Number(3.0)));
    }

    #[test]
    fn with_star() {
        let tokens = vec![Token::Number(3.0), Token::Star, Token::Number(5.0)];

        let mut parser = Parser::new(tokens);

        let result = parser.parse_term();

        assert_eq!(result, );

    }
}
