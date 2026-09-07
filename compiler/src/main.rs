use lexer::pub_lex;

fn main() {
    let token = pub_lex("3 + 5 - 2");
    println!("{:?}", token)
}
