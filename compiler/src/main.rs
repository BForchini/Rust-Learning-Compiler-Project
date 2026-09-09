use lexer::lex;

fn main() {
    let token = lex("3 + 5 - 2");
    println!("{:?}", token);
}
