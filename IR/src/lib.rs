use syntax::{CalculatorError, Expr::Number, Instructions, Temp};
pub use syntax::{CalculatorError::IrError, Expr, Operator};

#[derive(Debug, PartialEq)]
pub struct Program {
    pub instructions: Vec<Instructions>,
    pub next_temp: usize,
    // Result if fail?
}

impl Program {
    pub fn new() -> Self {
        // Creates program
        Self {
            instructions: Vec::new(),
            next_temp: 0,
        }
    }

    fn new_temp(&mut self) -> Temp {
        // Counts an external counter
        let temp = self.next_temp;
        self.next_temp += 1;
        temp
    }

    pub fn generate_ir(&mut self, expr: &Expr) -> Result<Temp, CalculatorError> {
        match expr {
            Number(value) => {
                let destination = self.new_temp();
                self.instructions.push(Instructions::LoadConstant {
                    value: *value,
                    destination: destination,
                }); //need to emit this to ir
                Ok(destination)
            }
            Expr::Binary {
                left,
                operator,
                right,
            } => {
                let left_gen = self.generate_ir(left)?;
                let right_gen = self.generate_ir(right)?;
                match operator {
                    Operator::Addition => {
                        let destination = self.next_temp;
                        self.instructions.push(Instructions::Add {
                            left: left_gen,
                            right: right_gen,
                            destination: destination,
                        });
                        self.new_temp();
                        Ok(destination)
                    }
                    Operator::Subtraction => {
                        let destination = self.next_temp;
                        self.instructions.push(Instructions::Subtract {
                            left: left_gen,
                            right: right_gen,
                            destination: destination,
                        });
                        self.new_temp();
                        Ok(destination)
                    }
                    Operator::Division => {
                        let destination = self.next_temp;
                        self.instructions.push(Instructions::Divide {
                            left: left_gen,
                            right: right_gen,
                            destination: destination,
                        });
                        self.new_temp();
                        Ok(destination)
                    }
                    Operator::Multiplication => {
                        let destination = self.next_temp;
                        self.instructions.push(Instructions::Multiply {
                            left: left_gen,
                            right: right_gen,
                            destination: destination,
                        });
                        self.new_temp();
                        Ok(destination)
                    }
                }
            }
        }
    }
}

#[cfg(test)]
pub mod test {

    use super::*;

    #[test]
    pub fn basic_case() {}
}
