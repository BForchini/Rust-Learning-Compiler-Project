use lexer::lex;

fn main() {
    match lex("2 + 3") {
        Ok(tokens) => println!("{:?}", tokens),
        Err(error) => println!("{}", error),
    }
}
