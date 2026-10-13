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

fn run() -> Result<(), Box<dyn Error>> {
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
    use super::compile_source;

    #[test]
    fn compiler_main_wires_nested_function_pipeline() {
        let (result, assembly) =
            compile_source("fn add(a, b) = a + b; fn twice(x) = add(x, x); twice(4)").unwrap();

        assert_eq!(result, 8.0);
        assert!(assembly.contains("bl _twice\n"));
        assert!(assembly.contains("bl _add\n"));
    }
}
