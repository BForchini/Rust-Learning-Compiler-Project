use std::path::PathBuf;

use ir::Program;
use syntax::{BackendError, Instructions, Temp};

struct Arm64Backend {
    asm: String,
}

impl Arm64Backend {

    pub fn generate(&mut self, program: &Program) {
        for instruction in &program.instructions {
            self.emit_instruction(instruction);
        }
    }

    fn write_asm(&mut self, path: impl Into<PathBuf>,) -> Result<(), BackendError> {
        let path = path.into();
        std::fs::write(&path, self.create_asm()).map_err(|source| {
            BackendError::WriteAssembly { path, source }
        })
    }

    fn create_asm(&mut self) -> String {
        format!(".text\n{}", self.asm)
    }
    pub fn validate_temp(temp: Temp) -> Result<(), BackendError> {
        if temp > 31 {
            Err(BackendError::InvalidRegister { temp })
        } else {
            Ok(())
        }
    }

    fn emit_instruction(&mut self, instruction: &Instructions) {
        match instruction {
            Instructions::LoadConstant { value, destination } => {
                self.emit_load_constant(*value, *destination);
            }
            Instructions::Add {
                left,
                right,
                destination,
            } => {
                self.emit_add(*left, *right, *destination);
            }
            Instructions::Subtract {
                left,
                right,
                destination,
            } => {
                self.emit_sub(*left, *right, *destination);
            }
            Instructions::Multiply {
                left,
                right,
                destination,
            } => {
                self.emit_mul(*left, *right, *destination);
            }
            Instructions::Divide {
                left,
                right,
                destination,
            } => {
                self.emit_div(*left, *right, *destination);
            }
        }
    }

    fn emit_load_constant(&mut self, value: f64, destination: Temp) -> Result<(), BackendError> {
        for temp in [destination] {
            Self::validate_temp(temp)?;
        }
        let bits: u64 = value.to_bits();
        self.asm
            .push_str(&format!("LDR d{destination}, =0x{bits:16X}\n"));
        Ok(())
    }

    fn emit_add(&mut self, left: Temp, right: Temp, destination: Temp) -> Result<(), BackendError> {
        for temp in [left, right, destination] {
            Self::validate_temp(temp)?;
        }
        self.asm
            .push_str(&format!("fadd d{destination}, d{left}, d{right}\n"));
        Ok(())
    }

    fn emit_sub(&mut self, left: Temp, right: Temp, destination: Temp) -> Result<(), BackendError> {
        for temp in [left, right, destination] {
            Self::validate_temp(temp)?;
        }
        self.asm
            .push_str(&format!("fsub d{destination}, d{left}, d{right}\n"));
        Ok(())
    }

    fn emit_mul(&mut self, left: Temp, right: Temp, destination: Temp) -> Result<(), BackendError> {
        for temp in [left, right, destination] {
            Self::validate_temp(temp)?;
        }
        self.asm
            .push_str(&format!("fmul d{destination}, d{left}, d{right}\n"));
        Ok(())
    }

    fn emit_div(&mut self, left: Temp, right: Temp, destination: Temp) -> Result<(), BackendError> {
        for temp in [left, right, destination] {
            Self::validate_temp(temp)?;
        }
        self.asm
            .push_str(&format!("fdiv d{destination}, d{left}, d{right}\n"));
        Ok(())
    }
}

#[cfg(test)]
pub mod tests {

    use crate::Arm64Backend;
    use ir::{Expr::Number, Program};
    use syntax::{Expr, Operator};

    #[test]
    fn simple_emit_constant_code() {
        let mut program = Program::new();
        let mut backend = Arm64Backend { asm: String::new() };

        let expr = Box::new(Number(5.0));

        let result = program.generate_ir(&expr);
        assert_eq!(result, Ok(0));

        backend.generate(&program);

        assert_eq!(backend.asm, "LDR d0, =0x4014000000000000\n");
    }
    #[test]
    fn addition_emits_arm_code() {
        let mut program = Program::new();
        let mut backend = Arm64Backend { asm: String::new() };

        let expr = Expr::Binary {
            left: Box::new(Expr::Number(5.0)),
            operator: Operator::Addition,
            right: Box::new(Expr::Number(3.0)),
        };

        let result = program.generate_ir(&expr);
        assert_eq!(result, Ok(2));

        backend.generate(&program);

        assert_eq!(
            backend.asm,
            "LDR d0, =0x4014000000000000\nLDR d1, =0x4008000000000000\nfadd d2, d0, d1\n"
        );
    }
    #[test]
    fn subtraction_emits_arm_code() {
        let mut program = Program::new();
        let mut backend = Arm64Backend { asm: String::new() };

        let expr = Expr::Binary {
            left: Box::new(Expr::Number(5.0)),
            operator: Operator::Subtraction,
            right: Box::new(Expr::Number(3.0)),
        };

        let result = program.generate_ir(&expr);
        assert_eq!(result, Ok(2));

        backend.generate(&program);

        assert_eq!(
            backend.asm,
            "LDR d0, =0x4014000000000000\nLDR d1, =0x4008000000000000\nfsub d2, d0, d1\n"
        );
    }
    #[test]
    fn multiplication_emits_arm_code() {
        let mut program = Program::new();
        let mut backend = Arm64Backend { asm: String::new() };

        let expr = Expr::Binary {
            left: Box::new(Expr::Number(5.0)),
            operator: Operator::Multiplication,
            right: Box::new(Expr::Number(3.0)),
        };

        let result = program.generate_ir(&expr);
        assert_eq!(result, Ok(2));

        backend.generate(&program);

        assert_eq!(
            backend.asm,
            "LDR d0, =0x4014000000000000\nLDR d1, =0x4008000000000000\nfmul d2, d0, d1\n"
        );
    }
    #[test]
    fn division_emits_arm_code() {
        let mut program = Program::new();
        let mut backend = Arm64Backend { asm: String::new() };

        let expr = Expr::Binary {
            left: Box::new(Expr::Number(5.0)),
            operator: Operator::Division,
            right: Box::new(Expr::Number(3.0)),
        };

        let result = program.generate_ir(&expr);
        assert_eq!(result, Ok(2));

        backend.generate(&program);

        assert_eq!(
            backend.asm,
            "LDR d0, =0x4014000000000000\nLDR d1, =0x4008000000000000\nfdiv d2, d0, d1\n"
        );
    }
    #[test]
    fn addition_multiplication_emits_arm_code() {
        let mut program = Program::new();
        let mut backend = Arm64Backend { asm: String::new() };

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

        backend.generate(&program);

        assert_eq!(
            backend.asm,
            "LDR d0, =0x4014000000000000\nLDR d1, =0x4008000000000000\nfmul d2, d0, d1\nLDR d3, =0x4010000000000000\nfadd d4, d2, d3\n"
        );
    }
}
