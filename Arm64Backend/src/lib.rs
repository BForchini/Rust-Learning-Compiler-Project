use ir::{FunctionIr, Program};
use std::path::PathBuf;
use syntax::{Instructions, Temp};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum BackendError {
    #[error(
        "invalid temporary register d{temp} at instruction {instruction_index}: {instruction:?}"
    )]
    InvalidTemp {
        temp: Temp,
        instruction_index: usize,
        instruction: Instructions,
    },
    #[error("invalid floating-point argument index {index} at instruction {instruction_index}")]
    InvalidArgument {
        index: usize,
        instruction_index: usize,
    },
    #[error(
        "function {name} has {actual} arguments; ARM64 supports at most 8 floating-point arguments"
    )]
    TooManyArguments { name: String, actual: usize },
    #[error("Failed to write assembly to {path}: {source}")]
    CreateDirectory {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("failed to write assembly to {path}: {source}")]
    WriteAssembly {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
}
pub struct Arm64Backend {
    asm: String,
    function_asm: String,
    literal_pool: String,
    current_function: Option<String>,
}

impl Arm64Backend {
    pub fn new() -> Self {
        Self {
            asm: String::new(),
            function_asm: String::new(),
            literal_pool: String::new(),
            current_function: None,
        }
    }

    pub fn generate(&mut self, program: &Program) -> Result<(), BackendError> {
        self.asm.clear();
        self.function_asm.clear();
        self.literal_pool.clear();
        self.current_function = None;

        for (instruction_index, instruction) in program.instructions.iter().enumerate() {
            self.emit_instruction(instruction_index, instruction)?;
        }
        for function in &program.functions {
            self.emit_function(function)?;
        }

        Ok(())
    }

    pub fn generate_to_file(
        &mut self,
        program: &Program,
        path: impl Into<PathBuf> + std::convert::AsRef<std::path::Path>,
        result_temp: Temp,
    ) -> Result<(), BackendError> {
        self.generate(program)?;
        self.write_asm(path, result_temp)
    }

    pub fn write_asm(
        &self,
        path: impl Into<PathBuf> + std::convert::AsRef<std::path::Path>,
        result_temp: Temp,
    ) -> Result<(), BackendError> {
        let path = path.as_ref();

        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|source| BackendError::CreateDirectory {
                path: parent.to_path_buf(),
                source,
            })?;
        }

        std::fs::write(path, self.create_asm(result_temp)).map_err(|source| {
            BackendError::WriteAssembly {
                path: path.to_path_buf(),
                source,
            }
        })
    }

    pub fn create_asm(&self, result_temp: Temp) -> String {
        format!(
            ".text\n.globl _main\n.p2align 2\n_main:\nstp x29, x30, [sp, #-80]!\nstr d8, [sp, #16]\nstr d9, [sp, #24]\nstr d10, [sp, #32]\nstr d11, [sp, #40]\nstr d12, [sp, #48]\nstr d13, [sp, #56]\nstr d14, [sp, #64]\nstr d15, [sp, #72]\nmov x29, sp\n{}mov x0, #0\nfmov d0, d{result_temp}\nldr d8, [sp, #16]\nldr d9, [sp, #24]\nldr d10, [sp, #32]\nldr d11, [sp, #40]\nldr d12, [sp, #48]\nldr d13, [sp, #56]\nldr d14, [sp, #64]\nldr d15, [sp, #72]\nldp x29, x30, [sp], #80\nret\n{}\n.section __TEXT,__const\n{}",
            self.asm, self.function_asm, self.literal_pool,
        )
    }

    fn emit_function(&mut self, function: &FunctionIr) -> Result<(), BackendError> {
        let previous_asm = std::mem::take(&mut self.asm);
        self.current_function = Some(function.name.clone());
        self.asm.push_str(&format!(
            ".globl _{name}\n.p2align 2\n_{name}:\nstp x29, x30, [sp, #-80]!\nstr d8, [sp, #16]\nstr d9, [sp, #24]\nstr d10, [sp, #32]\nstr d11, [sp, #40]\nstr d12, [sp, #48]\nstr d13, [sp, #56]\nstr d14, [sp, #64]\nstr d15, [sp, #72]\nmov x29, sp\n",
            name = function.name
        ));

        for (instruction_index, instruction) in function.instructions.iter().enumerate() {
            if let Err(error) = self.emit_instruction(instruction_index, instruction) {
                self.current_function = None;
                self.asm = previous_asm;
                return Err(error);
            }
        }
        self.asm.push_str(&format!(
            "fmov d0, d{}\nldr d8, [sp, #16]\nldr d9, [sp, #24]\nldr d10, [sp, #32]\nldr d11, [sp, #40]\nldr d12, [sp, #48]\nldr d13, [sp, #56]\nldr d14, [sp, #64]\nldr d15, [sp, #72]\nldp x29, x30, [sp], #80\nret\n",
            function.result_temp
        ));
        self.function_asm.push_str(&self.asm);
        self.current_function = None;
        self.asm = previous_asm;
        Ok(())
    }

    fn emit_instruction(
        &mut self,
        instruction_index: usize,
        instruction: &Instructions,
    ) -> Result<(), BackendError> {
        let temps = match instruction {
            Instructions::LoadArgument { destination, .. }
            | Instructions::LoadConstant { destination, .. } => vec![*destination],
            Instructions::Add {
                left,
                right,
                destination,
            }
            | Instructions::Subtract {
                left,
                right,
                destination,
            }
            | Instructions::Multiply {
                left,
                right,
                destination,
            }
            | Instructions::Divide {
                left,
                right,
                destination,
            }
            | Instructions::GreaterThan {
                left,
                right,
                destination,
            }
            | Instructions::LessThan {
                left,
                right,
                destination,
            }
            | Instructions::Equality {
                left,
                right,
                destination,
            } => vec![*left, *right, *destination],
            Instructions::Call {
                arguments,
                destination,
                ..
            } => arguments
                .iter()
                .copied()
                .chain(std::iter::once(*destination))
                .collect(),
        };

        for temp in temps {
            if temp > 31 {
                return Err(BackendError::InvalidTemp {
                    temp,
                    instruction_index,
                    instruction: instruction.clone(),
                });
            }
        }

        match instruction {
            Instructions::LoadArgument { index, destination } => {
                if *index >= 8 {
                    return Err(BackendError::InvalidArgument {
                        index: *index,
                        instruction_index,
                    });
                }
                self.asm
                    .push_str(&format!("fmov d{destination}, d{index}\n"));
            }
            Instructions::LoadConstant { value, destination } => {
                self.emit_load_constant(*value, *destination)
            }
            Instructions::Add {
                left,
                right,
                destination,
            } => self.emit_add(*left, *right, *destination),
            Instructions::Subtract {
                left,
                right,
                destination,
            } => self.emit_sub(*left, *right, *destination),
            Instructions::Multiply {
                left,
                right,
                destination,
            } => self.emit_mul(*left, *right, *destination),
            Instructions::Divide {
                left,
                right,
                destination,
            } => self.emit_div(*left, *right, *destination),
            Instructions::GreaterThan {
                left,
                right,
                destination,
            } => self.emit_compare(*left, *right, *destination, "gt"),
            Instructions::LessThan {
                left,
                right,
                destination,
            } => self.emit_compare(*left, *right, *destination, "lt"),
            Instructions::Equality {
                left,
                right,
                destination,
            } => self.emit_compare(*left, *right, *destination, "eq"),
            Instructions::Call {
                name,
                arguments,
                destination,
            } => self.emit_call(name, arguments, *destination)?,
        }

        Ok(())
    }

    fn emit_load_constant(&mut self, value: f64, destination: Temp) {
        let bits: u64 = value.to_bits();
        let label = match &self.current_function {
            Some(function) => format!("L{function}_const{destination}"),
            None => format!("Lconst{destination}"),
        };
        self.asm.push_str(&format!(
            "adrp x16, {label}@PAGE\nldr d{destination}, [x16, {label}@PAGEOFF]\n"
        ));
        self.literal_pool
            .push_str(&format!(".p2align 3\n{label}:\n.quad 0x{bits:016X}\n"));
    }

    fn emit_call(
        &mut self,
        name: &str,
        arguments: &[Temp],
        destination: Temp,
    ) -> Result<(), BackendError> {
        if arguments.len() > 8 {
            return Err(BackendError::TooManyArguments {
                name: name.to_owned(),
                actual: arguments.len(),
            });
        }

        self.asm.push_str("sub sp, sp, #272\n");
        for register in 0..32 {
            self.asm
                .push_str(&format!("str d{register}, [sp, #{}]\n", register * 8));
        }
        for (index, argument) in arguments.iter().enumerate() {
            self.asm
                .push_str(&format!("ldr d{index}, [sp, #{}]\n", argument * 8));
        }
        self.asm.push_str(&format!("bl _{name}\n"));
        self.asm.push_str("str d0, [sp, #256]\n");
        for register in 0..32 {
            self.asm
                .push_str(&format!("ldr d{register}, [sp, #{}]\n", register * 8));
        }
        self.asm.push_str(&format!(
            "ldr d{destination}, [sp, #256]\nadd sp, sp, #272\n"
        ));
        Ok(())
    }

    fn emit_add(&mut self, left: Temp, right: Temp, destination: Temp) {
        self.asm
            .push_str(&format!("fadd d{destination}, d{left}, d{right}\n"));
    }

    fn emit_sub(&mut self, left: Temp, right: Temp, destination: Temp) {
        self.asm
            .push_str(&format!("fsub d{destination}, d{left}, d{right}\n"));
    }

    fn emit_mul(&mut self, left: Temp, right: Temp, destination: Temp) {
        self.asm
            .push_str(&format!("fmul d{destination}, d{left}, d{right}\n"));
    }

    fn emit_div(&mut self, left: Temp, right: Temp, destination: Temp) {
        self.asm
            .push_str(&format!("fdiv d{destination}, d{left}, d{right}\n"));
    }

    fn emit_compare(&mut self, left: Temp, right: Temp, destination: Temp, condition: &str) {
        self.asm
            .push_str(&format!("fcmp d{left}, d{right}\n"));
        self.asm
            .push_str(&format!("cset w16, {condition}\n"));
        self.asm
            .push_str(&format!("scvtf d{destination}, w16\n"));
    }
}

#[cfg(test)]
pub mod tests {

    use crate::{Arm64Backend, BackendError};
    use ir::{Expr::Number, Operator::Addition, Program};
    use std::process::Command;
    use syntax::{Expr, FunctionDefinition, Operator, SourceProgram};

    #[test]
    fn simple_emit_constant_code() {
        let mut program = Program::new();
        let mut backend = Arm64Backend::new();

        let expr = Box::new(Number(5.0));
        let result_temp = program.generate_ir(&expr).unwrap();
        backend.generate(&program).unwrap();

        assert_eq!(result_temp, 0);

        assert_eq!(
            backend.asm,
            "adrp x16, Lconst0@PAGE\nldr d0, [x16, Lconst0@PAGEOFF]\n"
        );
    }
    #[test]
    fn addition_emits_arm_code() {
        let mut program = Program::new();
        let mut backend = Arm64Backend::new();

        let expr = Expr::Binary {
            left: Box::new(Expr::Number(5.0)),
            operator: Operator::Addition,
            right: Box::new(Expr::Number(3.0)),
        };

        let result = program.generate_ir(&expr);
        assert_eq!(result, Ok(2));

        backend.generate(&program).unwrap();

        assert_eq!(
            backend.asm,
            "adrp x16, Lconst0@PAGE\nldr d0, [x16, Lconst0@PAGEOFF]\nadrp x16, Lconst1@PAGE\nldr d1, [x16, Lconst1@PAGEOFF]\nfadd d2, d0, d1\n"
        );
    }

    #[test]
    fn variable_assignment_emits_initializer_code() {
        let mut program = Program::new();
        let mut backend = Arm64Backend::new();
        let expr = Expr::Assignment {
            name: "x".into(),
            value: Box::new(Expr::Binary {
                left: Box::new(Expr::Number(4.0)),
                operator: Operator::Addition,
                right: Box::new(Expr::Number(3.0)),
            }),
        };

        let result_temp = program.generate_ir(&expr).unwrap();
        assert_eq!(result_temp, 2);
        backend.generate(&program).unwrap();

        assert_eq!(
            backend.asm,
            "adrp x16, Lconst0@PAGE\nldr d0, [x16, Lconst0@PAGEOFF]\nadrp x16, Lconst1@PAGE\nldr d1, [x16, Lconst1@PAGEOFF]\nfadd d2, d0, d1\n"
        );
    }

    #[test]
    fn nested_function_calls_emit_arm64_calls() {
        let source = SourceProgram {
            functions: vec![
                FunctionDefinition {
                    name: "add".into(),
                    parameters: vec!["a".into(), "b".into()],
                    body: Expr::Binary {
                        left: Box::new(Expr::Variable("a".into())),
                        operator: Operator::Addition,
                        right: Box::new(Expr::Variable("b".into())),
                    },
                },
                FunctionDefinition {
                    name: "twice".into(),
                    parameters: vec!["x".into()],
                    body: Expr::FunctionCall {
                        name: "add".into(),
                        arguments: vec![Expr::Variable("x".into()), Expr::Variable("x".into())],
                    },
                },
            ],
            expression: Expr::FunctionCall {
                name: "twice".into(),
                arguments: vec![Expr::Number(4.0)],
            },
        };
        let mut program = Program::new();
        let result_temp = program.generate_program_ir(&source).unwrap();
        let mut backend = Arm64Backend::new();

        backend.generate(&program).unwrap();
        let assembly = backend.create_asm(result_temp);

        assert!(assembly.contains("bl _twice\n"));
        assert!(assembly.contains("_add:\n"));
        assert!(assembly.contains("_twice:\n"));
        assert!(assembly.contains("bl _add\n"));
        assert!(assembly.contains("fmov d0, d"));
    }
    #[test]
    fn subtraction_emits_arm_code() {
        let mut program = Program::new();
        let mut backend = Arm64Backend::new();

        let expr = Expr::Binary {
            left: Box::new(Expr::Number(5.0)),
            operator: Operator::Subtraction,
            right: Box::new(Expr::Number(3.0)),
        };

        let result = program.generate_ir(&expr);
        assert_eq!(result, Ok(2));

        backend.generate(&program).unwrap();

        assert_eq!(
            backend.asm,
            "adrp x16, Lconst0@PAGE\nldr d0, [x16, Lconst0@PAGEOFF]\nadrp x16, Lconst1@PAGE\nldr d1, [x16, Lconst1@PAGEOFF]\nfsub d2, d0, d1\n"
        );
    }
    #[test]
    fn multiplication_emits_arm_code() {
        let mut program = Program::new();
        let mut backend = Arm64Backend::new();

        let expr = Expr::Binary {
            left: Box::new(Expr::Number(5.0)),
            operator: Operator::Multiplication,
            right: Box::new(Expr::Number(3.0)),
        };

        let result = program.generate_ir(&expr);
        assert_eq!(result, Ok(2));

        backend.generate(&program).unwrap();

        assert_eq!(
            backend.asm,
            "adrp x16, Lconst0@PAGE\nldr d0, [x16, Lconst0@PAGEOFF]\nadrp x16, Lconst1@PAGE\nldr d1, [x16, Lconst1@PAGEOFF]\nfmul d2, d0, d1\n"
        );
    }
    #[test]
    fn division_emits_arm_code() {
        let mut program = Program::new();
        let mut backend = Arm64Backend::new();

        let expr = Expr::Binary {
            left: Box::new(Expr::Number(5.0)),
            operator: Operator::Division,
            right: Box::new(Expr::Number(3.0)),
        };

        let result = program.generate_ir(&expr);
        assert_eq!(result, Ok(2));

        backend.generate(&program).unwrap();

        assert_eq!(
            backend.asm,
            "adrp x16, Lconst0@PAGE\nldr d0, [x16, Lconst0@PAGEOFF]\nadrp x16, Lconst1@PAGE\nldr d1, [x16, Lconst1@PAGEOFF]\nfdiv d2, d0, d1\n"
        );
    }
    #[test]
    fn addition_multiplication_emits_arm_code() {
        let mut program = Program::new();
        let mut backend = Arm64Backend::new();

        let expr = Expr::Binary {
            left: Box::new(Expr::Binary {
                left: Box::new(Expr::Number(5.0)),
                operator: Operator::Multiplication,
                right: Box::new(Expr::Number(3.0)),
            }),
            operator: Operator::Addition,
            right: Box::new(Expr::Number(4.0)),
        };
        let result = program.generate_ir(&expr);
        assert_eq!(result, Ok(4));

        backend.generate(&program).unwrap();

        assert_eq!(
            backend.asm,
            "adrp x16, Lconst0@PAGE\nldr d0, [x16, Lconst0@PAGEOFF]\nadrp x16, Lconst1@PAGE\nldr d1, [x16, Lconst1@PAGEOFF]\nfmul d2, d0, d1\nadrp x16, Lconst3@PAGE\nldr d3, [x16, Lconst3@PAGEOFF]\nfadd d4, d2, d3\n"
        );
    }

    #[test]
    fn multiplication_addition_emits_arm_code_brackets() {
        let mut program = Program::new();
        let mut backend = Arm64Backend::new();

        let expr = Expr::Binary {
            left: Box::new(Expr::Binary {
                left: Box::new(Expr::Number(5.0)),
                operator: Operator::Addition,
                right: Box::new(Expr::Number(3.0)),
            }),
            operator: Operator::Multiplication,
            right: Box::new(Expr::Number(4.0)),
        };
        let result = program.generate_ir(&expr);
        assert_eq!(result, Ok(4));

        backend.generate(&program).unwrap();

        assert_eq!(
            backend.asm,
            "adrp x16, Lconst0@PAGE\nldr d0, [x16, Lconst0@PAGEOFF]\nadrp x16, Lconst1@PAGE\nldr d1, [x16, Lconst1@PAGEOFF]\nfadd d2, d0, d1\nadrp x16, Lconst3@PAGE\nldr d3, [x16, Lconst3@PAGEOFF]\nfmul d4, d2, d3\n"
        );
    }

    #[test]
    fn simple_asm_code_generation_macos() -> Result<(), BackendError> {
        let mut program = Program::new();
        let expr = Box::new(Number(5.0));

        let mut backend = Arm64Backend::new();

        let result_temp = program.generate_ir(&expr).unwrap();
        assert_eq!(result_temp, 0);

        backend.generate(&program).unwrap();

        assert_eq!(
            backend.asm,
            "adrp x16, Lconst0@PAGE\nldr d0, [x16, Lconst0@PAGEOFF]\n"
        );
        assert!(
            backend
                .create_asm(result_temp)
                .starts_with(".text\n.globl _main\n.p2align 2\n_main:\n")
        );
        let assembly_path = "target/aarch64-apple-darwin/debug/asm/simple_test.s";
        let executable_path = "target/aarch64-apple-darwin/debug/asm/test_simple";
        backend.generate_to_file(&program, assembly_path, result_temp)?;

        let output = Command::new("clang")
            .args([assembly_path, "-o", executable_path])
            .output()
            .expect("failed to run clang");
        assert!(
            output.status.success(),
            "clang failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );

        Ok(())
    }
    #[test]
    fn simple_addition_asm_code_generation_macos() -> Result<(), BackendError> {
        let mut program = Program::new();
        let expr = Expr::Binary {
            left: Box::new(Number(5.0)),
            operator: Addition,
            right: Box::new(Number(3.0)),
        };

        let mut backend = Arm64Backend::new();

        let result_temp = program.generate_ir(&expr).unwrap();
        assert_eq!(result_temp, 2);

        backend.generate(&program).unwrap();

        assert_eq!(
            backend.asm,
            "adrp x16, Lconst0@PAGE\nldr d0, [x16, Lconst0@PAGEOFF]\nadrp x16, Lconst1@PAGE\nldr d1, [x16, Lconst1@PAGEOFF]\nfadd d2, d0, d1\n"
        );
        let assembly_path = "target/aarch64-apple-darwin/debug/asm/simple_addition_test.s";
        let executable_path = "target/aarch64-apple-darwin/debug/asm/test_simple_addition";
        backend.generate_to_file(&program, assembly_path, result_temp)?;

        let output = Command::new("clang")
            .args([assembly_path, "-o", executable_path])
            .output()
            .expect("failed to run clang");
        assert!(
            output.status.success(),
            "clang failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );

        Ok(())
    }
    #[test]
    fn addition_multiplication_asm_code_generation_macos() -> Result<(), BackendError> {
        let mut program = Program::new();
        let mut backend = Arm64Backend::new();

        let expr = Expr::Binary {
            left: Box::new(Expr::Binary {
                left: Box::new(Expr::Number(5.0)),
                operator: Operator::Multiplication,
                right: Box::new(Expr::Number(3.0)),
            }),
            operator: Operator::Addition,
            right: Box::new(Expr::Number(4.0)),
        };
        let result_temp = program.generate_ir(&expr).unwrap();
        assert_eq!(result_temp, 4);

        backend.generate(&program).unwrap();

        assert_eq!(
            backend.asm,
            "adrp x16, Lconst0@PAGE\nldr d0, [x16, Lconst0@PAGEOFF]\nadrp x16, Lconst1@PAGE\nldr d1, [x16, Lconst1@PAGEOFF]\nfmul d2, d0, d1\nadrp x16, Lconst3@PAGE\nldr d3, [x16, Lconst3@PAGEOFF]\nfadd d4, d2, d3\n"
        );

        let assembly_path = "target/aarch64-apple-darwin/debug/asm/simple_multadd_test.s";
        let executable_path = "target/aarch64-apple-darwin/debug/asm/test_simple_multadd";
        backend.generate_to_file(&program, assembly_path, result_temp)?;

        let output = Command::new("clang")
            .args([assembly_path, "-o", executable_path])
            .output()
            .expect("failed to run clang");
        assert!(
            output.status.success(),
            "clang failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );

        Ok(())
    }
    #[test]
    fn simple_addition_asm_code_generation_linux() -> Result<(), BackendError> {
        let mut program = Program::new();
        let expr = Expr::Binary {
            left: Box::new(Number(5.0)),
            operator: Addition,
            right: Box::new(Number(3.0)),
        };

        let mut backend = Arm64Backend::new();

        let result_temp = program.generate_ir(&expr).unwrap();
        assert_eq!(result_temp, 2);

        backend.generate(&program).unwrap();

        assert_eq!(
            backend.asm,
            "adrp x16, Lconst0@PAGE\nldr d0, [x16, Lconst0@PAGEOFF]\nadrp x16, Lconst1@PAGE\nldr d1, [x16, Lconst1@PAGEOFF]\nfadd d2, d0, d1\n"
        );
        backend.generate_to_file(
            &program,
            "target/aarch64-unknown-linux-gnu/debug/asm/expression.s",
            result_temp,
        )?;

        Ok(())
    }
}
