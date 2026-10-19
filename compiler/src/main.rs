use std::{
    error::Error,
    fs,
    path::PathBuf,
    process::{self, Command},
};

fn main() {
    if let Err(error) = run() {
        eprintln!("{error}");
        process::exit(1);
    }
}

fn verify_language_features() -> Result<(), Box<dyn Error>> {
    let tokens = lexer::lex("if x < 5 == 3")?;
    assert_eq!(
        tokens,
        vec![
            syntax::Token::If,
            syntax::Token::Identifier("x".to_string()),
            syntax::Token::LessThan,
            syntax::Token::Number(5.0),
            syntax::Token::EqualsEquals,
            syntax::Token::Number(3.0),
        ]
    );

    let comparison_tokens = lexer::lex("x < 5 == 3")?;
    let mut parser = parser::Parser::new(comparison_tokens);
    let parsed = parser.parse()?;
    assert_eq!(
        parsed,
        syntax::Expr::Binary {
            left: Box::new(syntax::Expr::Binary {
                left: Box::new(syntax::Expr::Variable("x".to_string())),
                operator: syntax::Operator::LessThan,
                right: Box::new(syntax::Expr::Number(5.0)),
            }),
            operator: syntax::Operator::Equality,
            right: Box::new(syntax::Expr::Number(3.0)),
        }
    );

    let mut evaluator = evaluator::Evaluator::new();
    evaluator.evaluate(syntax::Expr::Assignment {
        name: "x".to_string(),
        value: Box::new(syntax::Expr::Number(4.0)),
    })?;
    let comparison = evaluator.evaluate(syntax::Expr::Binary {
        left: Box::new(syntax::Expr::Binary {
            left: Box::new(syntax::Expr::Variable("x".to_string())),
            operator: syntax::Operator::LessThan,
            right: Box::new(syntax::Expr::Number(5.0)),
        }),
        operator: syntax::Operator::Equality,
        right: Box::new(syntax::Expr::Number(1.0)),
    })?;
    assert_eq!(comparison, 1.0);

    let source = "fn is_small(x) = x < 5; is_small(3)";
    let (result, assembly) = compile_source(source)?;
    assert_eq!(result, 1.0);
    assert!(assembly.contains("fcmp d"));

    let (result, assembly) = compile_source("fn add(a, b) = a + b; fn twice(x) = add(x, x); twice(4)")?;
    assert_eq!(result, 8.0);
    assert!(assembly.contains("bl _twice\n"));
    assert!(assembly.contains("bl _add\n"));

    Ok(())
}

fn run() -> Result<(), Box<dyn Error>> {
    verify_language_features()?;

    let source = "fn add(a, b) = a + b; fn twice(x) = add(x, x); twice(4)";
    let (result, assembly) = compile_source(source)?;
    let output_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../target/compiler-output");
    fs::create_dir_all(&output_dir)?;

    let assembly_path = output_dir.join("program.s");
    let executable_path = output_dir.join("program");
    fs::write(&assembly_path, assembly.as_bytes())?;

    let clang_output = Command::new("clang")
        .arg(&assembly_path)
        .arg("-o")
        .arg(&executable_path)
        .output()?;
    if !clang_output.status.success() {
        return Err(format!(
            "clang failed to compile {}: {}",
            assembly_path.display(),
            String::from_utf8_lossy(&clang_output.stderr)
        )
        .into());
    }

    println!("Result: {result}");
    println!("Generated assembly: {}", assembly_path.display());
    println!("Compiled executable: {}", executable_path.display());
    Ok(())
}

fn compile_source(source: &str) -> Result<(f64, String), Box<dyn Error>> {
    let tokens = lexer::lex(source)?;
    let mut parser = parser::Parser::new(tokens);
    let program: syntax::SourceProgram = parser.parse_program()?;

    let result = evaluator::Evaluator::new().evaluate_program(&program)?;

    let mut ir = ir::Program::new();
    let result_temp = ir.generate_program_ir(&program)?;
    let mut backend = Arm64Backend::Arm64Backend::new();
    backend.generate(&ir)?;
    let assembly = backend.create_asm(result_temp);

    Ok((result, assembly))
}

#[cfg(test)]
mod tests {
    use super::{compile_source, verify_language_features};

    #[test]
    fn compiler_main_wires_nested_function_pipeline() {
        let (result, assembly) =
            compile_source("fn add(a, b) = a + b; fn twice(x) = add(x, x); twice(4)").unwrap();

        assert_eq!(result, 8.0);
        assert!(assembly.contains("bl _twice\n"));
        assert!(assembly.contains("bl _add\n"));
    }

    #[test]
    fn compiler_verifies_keyword_and_comparison_syntax() {
        verify_language_features().unwrap();
    }
}
