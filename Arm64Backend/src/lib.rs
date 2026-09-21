use ir::Program;
use syntax::{Instructions, Temp};
/*
#[derive(Debug, PartialEq)]
pub struct Program {
    pub instructions: Vec<Instructions>,
    next_temp: usize,
    // Result if fail?
}

    program.instructions,
    vec![Instructions::LoadConstant {
        value: 5.0,
        destination: 0
    }]
*/

struct Arm64Backend {
    asm: String,
}

impl Arm64Backend {
    pub fn generate(&mut self, program: &Program) {
        for instruction in &program.instructions {
            self.emit_instruction(instruction);
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

    fn emit_load_constant(&mut self, value: f64, destination: Temp) {
        /*
        _emit_load_constant:
        sub     sp, sp, #16
        str     w0, [sp, #12]
        add     sp, sp, #16
        ret
        */
    }

    fn emit_add(&mut self, left: Temp, right: Temp, destination: Temp) {
        self.asm
            .push_str(&format!("fadd d{destination}, d{left}, d{right} "));
    }
    fn emit_sub(&mut self, left: Temp, right: Temp, destination: Temp) {
        self.asm
            .push_str(&format!("fsub d{destination}, d{left}, d{right} "));
    }
    fn emit_mul(&mut self, left: Temp, right: Temp, destination: Temp) {
        self.asm
            .push_str(&format!("fmul d{destination}, d{left}, d{right} "));
    }
    fn emit_div(&mut self, left: Temp, right: Temp, destination: Temp) {
        self.asm
            .push_str(&format!("fdiv d{destination}, d{left}, d{right} "));
    }
}

// i need to have a string final depending on the operators present idk deal with tommorow

#[cfg(test)]
pub mod tests {

    use crate::Arm64Backend;
    use ir::Program;

    #[test]
    fn simple_load_constant() {
        let program = Program::new();
        let mut backend = Arm64Backend { asm: String::new() };

        backend.generate(&program);

        assert!(backend.asm.is_empty());
    }
}
