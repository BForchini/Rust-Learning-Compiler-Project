use syntax::{
    CalculatorError::{self},
    Token,
};

pub fn lex(input: &str) -> Result<Vec<Token>, CalculatorError> {
    let trimmed = input.trim();
    tokenize(trimmed)
}

fn tokenize(trimmed: &str) -> Result<Vec<Token>, CalculatorError> {
    let mut chars = trimmed.chars().peekable();
    let mut tokens: Vec<Token> = Vec::new();

    while let Some(c) = chars.next() {
        if c.is_whitespace() {
            continue;
        }

        if c == '"' {
            let mut value = String::new();
            loop {
                let Some(next) = chars.next() else {
                    return Err(CalculatorError::InputInvalid);
                };

                if next == '"' {
                    break;
                }

                if next == '\\' {
                    let Some(escaped) = chars.next() else {
                        return Err(CalculatorError::InputInvalid);
                    };
                    match escaped {
                        '"' | '\\' => value.push(escaped),
                        'n' => value.push('\n'),
                        'r' => value.push('\r'),
                        't' => value.push('\t'),
                        _ => value.push(escaped),
                    }
                    continue;
                }

                value.push(next);
            }
            tokens.push(Token::String(value));
            continue;
        }

        if c.is_ascii_digit() {
            let mut number = String::new();
            number.push(c);
            while let Some(&next) = chars.peek() {
                if next.is_ascii_digit() || next == '.' {
                    number.push(next);
                    chars.next();
                } else {
                    break;
                }
            }
            tokens.push(Token::Number(number.parse::<f64>()?));
            continue;
        }

        if c.is_ascii_alphabetic() || c == '_' {
            let mut identifier = String::new();
            identifier.push(c);
            while let Some(&next) = chars.peek() {
                if next.is_ascii_alphanumeric() || next == '_' {
                    identifier.push(next);
                    chars.next();
                } else {
                    break;
                }
            }
            let token = match identifier.as_str() {
                "fn" => Token::Fn,
                "if" => Token::If,
                "else" => Token::Else,
                "for" => Token::For,
                "while" => Token::While,
                _ => Token::Identifier(identifier),
            };
            tokens.push(token);
            continue;
        }

        if let Some(oper) = parse_operator(c, &mut chars) {
            tokens.push(oper);
            continue;
        }

        return Err(CalculatorError::InputInvalid);
    }

    Ok(tokens)
}

fn parse_operator(
    c: char,
    chars: &mut std::iter::Peekable<std::str::Chars<'_>>,
) -> Option<Token> {
    match c {
        '=' => {
            if matches!(chars.peek(), Some('=')) {
                chars.next();
                Some(Token::EqualsEquals)
            } else {
                Some(Token::Equals)
            }
        }
        '>' => Some(Token::GreaterThan),
        '<' => Some(Token::LessThan),
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

fn push_number(
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
        Comma, Equals, EqualsEquals, Fn, Identifier, If, LeftParen, LessThan, Minus, Number,
        Plus, RightParen, Semicolon, Star,
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

    #[test]
    fn keywords_and_boolean_operators() {
        let result = lex("if x < 5 == 3");

        assert!(result.is_ok());

        let tokens = result.unwrap();

        assert_eq!(
            tokens,
            vec![If, Identifier("x".into()), LessThan, Number(5.0), EqualsEquals, Number(3.0)]
        );
    }

    #[test]
    fn strings() {
        let result = lex("\"hello\"");

        assert!(result.is_ok());

        let tokens = result.unwrap();

        assert_eq!(tokens, vec![Token::String("hello".into())]);
    }
}
