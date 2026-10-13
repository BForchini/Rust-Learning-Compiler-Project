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
    let mut current_identifier = String::new();
    let mut tokens: Vec<Token> = Vec::new();

    for c in trimmed.chars() {
        if c.is_ascii_alphanumeric() || c == '_' {
            if !current_identifier.is_empty() || c.is_ascii_alphabetic() || c == '_' {
                push_number(&mut current_number, &mut tokens)?;
                current_identifier.push(c);
            } else {
                push_identifier(&mut current_identifier, &mut tokens);
                current_number.push(c);
            }
        } else if c.is_numeric() {
            current_number.push(c);
        } else if let Some(oper) = parse_operator(c) {
            push_number(&mut current_number, &mut tokens)?;
            push_identifier(&mut current_identifier, &mut tokens);
            tokens.push(oper);
        } else if c.is_whitespace() {
            push_number(&mut current_number, &mut tokens)?;
            push_identifier(&mut current_identifier, &mut tokens);
        } else {
            return Err(CalculatorError::InputInvalid);
        }
    }

    push_number(&mut current_number, &mut tokens)?;
    push_identifier(&mut current_identifier, &mut tokens);

    Ok(tokens)
}

fn parse_operator(c: char) -> Option<Token> {
    match c {
        '=' => Some(Token::Equals),
        ',' => Some(Token::Comma),
        ';' => Some(Token::Semicolon),
        '+' => Some(Token::Plus),
        '-' => Some(Token::Minus),
        '*' => Some(Token::Star),
        '/' => Some(Token::Slash),
        '(' => Some(Token::LeftParen),
        ')' => Some(Token::RightParen),
        _ => None,
    }
}

fn push_identifier(current_identifier: &mut String, tokens: &mut Vec<Token>) {
    if !current_identifier.is_empty() {
        let identifier = std::mem::take(current_identifier);
        if identifier == "fn" {
            tokens.push(Token::Fn);
        } else {
            tokens.push(Token::Identifier(identifier));
        }
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
    use syntax::Token::{
        Comma, Equals, Fn, Identifier, LeftParen, Minus, Number, Plus, RightParen, Semicolon, Star,
    };

    #[test]
    fn function_definitions_and_calls() {
        assert_eq!(
            lex("fn add(a, b) = a + b; add(4, 3)"),
            Ok(vec![
                Fn,
                Identifier("add".into()),
                LeftParen,
                Identifier("a".into()),
                Comma,
                Identifier("b".into()),
                RightParen,
                Equals,
                Identifier("a".into()),
                Plus,
                Identifier("b".into()),
                Semicolon,
                Identifier("add".into()),
                LeftParen,
                Number(4.0),
                Comma,
                Number(3.0),
                RightParen,
            ])
        );
        assert_eq!(lex(";"), Ok(vec![Semicolon]));
    }

    #[test]
    fn variable_assignment() {
        assert_eq!(
            lex("x = 4 + 3"),
            Ok(vec![
                Identifier("x".into()),
                Equals,
                Number(4.0),
                Plus,
                Number(3.0),
            ])
        );
    }

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
    fn brackets() {
        let result = lex("(3)");

        assert!(result.is_ok());

        let tokens = result.unwrap();

        assert_eq!(tokens, vec![LeftParen, Number(3.0), RightParen])
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
