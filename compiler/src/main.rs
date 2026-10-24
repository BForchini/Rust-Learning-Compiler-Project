use std::{
    error::Error,
    fs, io,
    path::PathBuf,
    process::{self, Command},
};

fn main() {
    if let Err(error) = run() {
        eprintln!("{error}");
        process::exit(1);
    }
}

#[cfg(test)]
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

    let (result, assembly) =
        compile_source("fn add(a, b) = a + b; fn twice(x) = add(x, x); twice(4)")?;
    assert_eq!(result, 8.0);
    assert!(assembly.contains("bl _twice\n"));
    assert!(assembly.contains("bl _add\n"));

    Ok(())
}

fn run() -> Result<(), Box<dyn Error>> {
    match std::env::args_os().nth(1) {
        Some(argument) if argument == "--repl" => run_repl(),
        Some(path) => run_source_file(PathBuf::from(path)),
        None => run_compiler_demo(),
    }
}

fn run_source_file(path: PathBuf) -> Result<(), Box<dyn Error>> {
    let source = fs::read_to_string(&path)?;
    let tokens = lexer::lex(&source)?;
    let mut parser = parser::Parser::new(tokens);
    let program = parser.parse_program()?;

    let stdin = io::stdin();
    let stdout = io::stdout();
    evaluator::Evaluator::new().evaluate_program_with_io(
        &program,
        &mut stdin.lock(),
        &mut stdout.lock(),
    )?;
    Ok(())
}

fn run_repl() -> Result<(), Box<dyn Error>> {
    use std::io::BufRead;

    let stdin = io::stdin();
    let mut evaluator = evaluator::Evaluator::new();
    for line in stdin.lock().lines() {
        let line = line?;
        let source = line.trim();
        if source.is_empty() {
            continue;
        }
        if source == ":quit" {
            break;
        }

        match evaluate_source(&mut evaluator, source) {
            Ok(result) => println!("{result}"),
            Err(error) => eprintln!("Error: {error}"),
        }
    }
    Ok(())
}

fn run_compiler_demo() -> Result<(), Box<dyn Error>> {
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

fn evaluate_source(
    evaluator: &mut evaluator::Evaluator,
    source: &str,
) -> Result<f64, syntax::CalculatorError> {
    let tokens = lexer::lex(source)?;
    let mut parser = parser::Parser::new(tokens);
    let program: syntax::SourceProgram = parser.parse_program()?;
    evaluator.evaluate_program(&program)
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
    use super::{compile_source, evaluate_source, verify_language_features};
    use std::io::Cursor;

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

    #[test]
    fn calculator_evaluates_input_and_keeps_variables_between_entries() {
        let mut evaluator = evaluator::Evaluator::new();

        assert_eq!(
            evaluate_source(&mut evaluator, "total = 4 + 3 * 2"),
            Ok(10.0)
        );
        assert_eq!(evaluate_source(&mut evaluator, "total - 1"), Ok(9.0));
    }

    #[test]
    fn calculator_evaluates_function_definitions_and_calls() {
        let mut evaluator = evaluator::Evaluator::new();

        assert_eq!(
            evaluate_source(&mut evaluator, "fn add(a, b) = a + b; add(4, 3)"),
            Ok(7.0)
        );
    }

    #[test]
    fn in_language_calculator_reads_operands_and_selects_operations() {
        let source = include_str!("../../examples/calculator.calc");
        let tokens = lexer::lex(source).unwrap();
        let mut parser = parser::Parser::new(tokens);
        let program = parser.parse_program().unwrap();
        let output_suffix =
            "Left operand:\nRight operand:\nOperation: 1=add, 2=subtract, 3=multiply, 4=divide\n";

        for (input, expected) in [
            ("9\n3\n1\n", "12\n"),
            ("9\n3\n2\n", "6\n"),
            ("9\n3\n3\n", "27\n"),
            ("9\n3\n4\n", "3\n"),
            ("9\n0\n4\n", "Cannot divide by zero.\n"),
            (
                "9\n3\n5\n",
                "Unknown operation; enter a code from 1 to 4.\n",
            ),
        ] {
            let mut evaluator = evaluator::Evaluator::new();
            let mut input = Cursor::new(input.as_bytes());
            let mut output = Vec::new();

            evaluator
                .evaluate_program_with_io(&program, &mut input, &mut output)
                .unwrap();

            assert_eq!(
                String::from_utf8(output).unwrap(),
                format!("{output_suffix}{expected}")
            );
        }
    }
}
