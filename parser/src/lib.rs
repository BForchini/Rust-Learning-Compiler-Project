use syntax::CalculatorError::{ParseError};
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

    fn parse_factor(&mut self) -> Result<Expr, CalculatorError> { // This is for numbers
        let token = self.tokens.get(self.position).clone();
        match token {
            Some(Token::Number(value)) => {
                self.position += 1;
                Ok(Expr::Number(*value))
            }
            _ => {
                Err(ParseError)
            }
        }
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.position)
    }

    fn parse_term(&mut self) -> Result<Expr, CalculatorError> { // This is for multiplication and division
        let mut left = self.parse_factor()?;
        loop{
            match self.peek() {
                Some(Token::Star) => {
                    self.position += 1;
                    let right = self.parse_factor()?;
                    left = Expr::Binary { left: Box::new(left), operator: Operator::Multiplication, right: Box::new(right) };

                },
                Some(Token::Slash) => {
                    self.position += 1;
                    let right = self.parse_factor()?;
                    left = Expr::Binary { left: Box::new(left), operator: Operator::Division, right: Box::new(right) };
                },
                _ => break,
            }
        }
        Ok(left)
    }

    fn parse_expression(&mut self) -> Result<Expr, CalculatorError> { // This is for addition and subtraction
        let mut left = self.parse_term()?;
        loop{
            match self.peek() {
                Some(Token::Plus) => {
                    self.position += 1;
                    let right = self.parse_term()?;
                    left = Expr::Binary { left: Box::new(left), operator: Operator::Addition, right: Box::new(right) };
                },
                Some(Token::Minus) => {
                    self.position += 1;
                    let right = self.parse_term()?;
                    left = Expr::Binary { left: Box::new(left), operator: Operator::Subtraction, right: Box::new(right) };
                },
                _ => break,
                
            }
        }
        Ok(left)

    }
}


#[cfg(test)]
pub mod tests {

    use syntax::{{Expr::{self, Number}, Operator, Token::{self, Star}},{Operator::Addition}};

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

        assert_eq!(result, Ok(Expr::Binary { left: Box::new(Expr::Number(3.0)), operator: Operator::Multiplication, right: Box::new(Expr::Number(5.0)) }));

    }

    #[test]
    fn multi_number() {
        let tokens = vec![Token::Number(3.0), Token::Star, Token::Number(5.0), Token::Star, Token::Number(4.0)];

        let mut parser = Parser::new(tokens);

        let result = parser.parse_term();

        assert_eq!(result, Ok(Expr::Binary{left: Box::new(Expr::Binary { left: Box::new(Expr::Number(3.0)), operator: Operator::Multiplication, right: Box::new(Expr::Number(5.0)) }) , operator: Operator::Multiplication, right: Box::new(Expr::Number(4.0)) }));
    }

    #[test]
    fn division() {
        let tokens = vec![Token::Number(3.0), Token::Slash, Token::Number(5.0)];

        let mut parser = Parser::new(tokens);

        let result = parser.parse_term();

        assert_eq!(result, Ok(Expr::Binary { left: Box::new(Number(3.0)), operator: Operator::Division, right: Box::new(Number(5.0)) }));
    }

    #[test]
    fn multi_operator() {
        let tokens = vec![Token::Number(3.0), Token::Slash, Token::Number(5.0), Token::Star, Token::Number(4.0)];

        let mut parser = Parser::new(tokens);

        let result = parser.parse_term();

        assert_eq!(result, Ok(Expr::Binary{left: Box::new(Expr::Binary { left: Box::new(Expr::Number(3.0)), operator: Operator::Division, right: Box::new(Expr::Number(5.0)) }) , operator: Operator::Multiplication, right: Box::new(Expr::Number(4.0)) }));
    }

    #[test]
    fn addition() {
        let tokens = vec![Token::Number(3.0), Token::Plus, Token::Number(5.0)];

        let mut parser = Parser::new(tokens);

        let result = parser.parse_expression();

        assert_eq!(result, Ok(Expr::Binary { left: Box::new(Number(3.0)), operator: Operator::Addition, right: Box::new(Number(5.0)) }));
    }

    #[test]
    fn addition_and_multiplication() {
        let tokens = vec![Token::Number(3.0), Token::Star, Token::Number(5.0), Token::Plus, Token::Number(4.0)];

        let mut parser = Parser::new(tokens);

        let result = parser.parse_expression();

        assert_eq!(result, Ok(Expr::Binary{left: Box::new(Expr::Binary { left: Box::new(Expr::Number(3.0)), operator: Operator::Multiplication, right: Box::new(Expr::Number(5.0)) }) , operator: Operator::Addition, right: Box::new(Expr::Number(4.0)) }));
    }

    #[test]
    fn addition_and_addition() {
        let tokens = vec![Token::Number(3.0), Token::Plus, Token::Number(5.0), Token::Plus, Token::Number(4.0)];

        let mut parser = Parser::new(tokens);

        let result = parser.parse_expression();

        assert_eq!(result, Ok(Expr::Binary{left: Box::new(Expr::Binary { left: Box::new(Expr::Number(3.0)), operator: Operator::Addition, right: Box::new(Expr::Number(5.0)) }) , operator: Operator::Addition, right: Box::new(Expr::Number(4.0)) }));
    }

    #[test]
    fn addition_and_multiplication_reversed() {
        let tokens = vec![Token::Number(3.0), Token::Plus, Token::Number(5.0), Token::Star, Token::Number(4.0)];

        let mut parser = Parser::new(tokens);

        let result = parser.parse_expression();

        assert_eq!(result, Ok(Expr::Binary{left: Box::new(Number(3.0)) , operator: Operator::Addition, right: Box::new(Expr::Binary { left: Box::new(Expr::Number(5.0)), operator: Operator::Multiplication, right: Box::new(Expr::Number(4.0)) })}));
    }

}
