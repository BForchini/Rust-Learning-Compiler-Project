use syntax::{
    CalculatorError::{self},
    Token,
};


pub fn lex(input: &str) -> Result<Vec<Token>, CalculatorError> {
    let trimmed = input.trim();
    tokenize(trimmed)
}

fn tokenize(trimmed: &str) -> Result<Vec<Token>, CalculatorError> {
    let mut current_number = String::new();
    let mut tokens: Vec<Token> = Vec::new();

    for c in trimmed.chars() {
        if c.is_numeric() {
            current_number.push(c);
        } else if let Some(oper) = parse_operator(c) {
            if !current_number.is_empty() {
                let value = current_number.parse::<f64>()?;
                tokens.push(Token::Number(value));
                current_number = String::new();
                tokens.push(oper);
            }
        } else if !c.is_whitespace() {
            return Err(CalculatorError::InputInvalid);
        }
    }

    if !current_number.is_empty() {
        push_number(&mut current_number, &mut tokens)?;
    }
    
    Ok(tokens)
}

fn parse_operator(c: char) -> Option<Token> {
    match c {
        '+' => Some(Token::Plus),
        '-' => Some(Token::Minus),
        '*' => Some(Token::Star),
        '/' => Some(Token::Slash),
        _ => None,
    }
}

fn push_number(
    //Function is for the final term of the string.
    current_numbers: &mut String,
    tokens: &mut Vec<Token>,
) -> Result<(), CalculatorError> {
    if !current_numbers.is_empty() {
        let value = current_numbers.parse::<f64>()?;
        tokens.push(Token::Number(value));
        current_numbers.clear();
    }
    Ok(())
}

#[cfg(test)]
pub mod tests {

    use super::*;
    use syntax::Token::{Minus, Number, Plus, Star};

    #[test]
    fn no_white_space() {
        let result = lex("3+5");

        assert!(result.is_ok());

        let tokens = result.unwrap();

        assert_eq!(tokens, vec![Number(3.0), Plus, Number(5.0)]);
    }

    #[test]
    fn one_gap_whitespace() {
        let result = lex("3 + 5");

        assert!(result.is_ok());

        let tokens = result.unwrap();

        assert_eq!(tokens, vec![Number(3.0), Plus, Number(5.0)]);
    }

    #[test]
    fn multi_integer_star() {
        let result = lex("123 * 456");

        assert!(result.is_ok());

        let tokens = result.unwrap();

        assert_eq!(tokens, vec![Number(123.0), Star, Number(456.0)]);
    }

    #[test]
    fn number_parse_error() {
        let mut current_number = String::from("not-a-number");
        let mut tokens = Vec::new();

        let result = push_number(&mut current_number, &mut tokens);

        assert!(matches!(
            result,
            Err(syntax::CalculatorError::NumberParse(_))
        ));
    }

    #[test]
    fn multi_operator() {
        let result = lex("3 + 5 * 2");

        assert!(result.is_ok());

        let tokens = result.unwrap();

        assert_eq!(
            tokens,
            vec![Number(3.0), Plus, Number(5.0), Star, Number(2.0)]
        );
    }

    #[test]
    fn negative_num() {
        let result = lex("3 - 2");

        assert!(result.is_ok());

        let tokens = result.unwrap();

        assert_eq!(tokens, vec![Number(3.0), Minus, Number(2.0)]);
    }
}
