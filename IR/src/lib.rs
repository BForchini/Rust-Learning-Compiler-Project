use syntax::{CalculatorError, Expr::Number, Instructions, Temp};
pub use syntax::{CalculatorError::IrError, Expr, Operator};

#[derive(Debug, PartialEq)]
pub struct Program {
    pub instructions: Vec<Instructions>,
    next_temp: usize,
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
                        let destination = self.new_temp();
                        self.instructions.push(Instructions::Add {
                            left: left_gen,
                            right: right_gen,
                            destination: destination,
                        });
                        Ok(destination)
                    }
                    Operator::Subtraction => {
                        let destination = self.new_temp();
                        self.instructions.push(Instructions::Subtract {
                            left: left_gen,
                            right: right_gen,
                            destination: destination,
                        });
                        Ok(destination)
                    }
                    Operator::Division => {
                        let destination = self.new_temp();
                        self.instructions.push(Instructions::Divide {
                            left: left_gen,
                            right: right_gen,
                            destination: destination,
                        });
                        Ok(destination)
                    }
                    Operator::Multiplication => {
                        let destination = self.new_temp();
                        self.instructions.push(Instructions::Multiply {
                            left: left_gen,
                            right: right_gen,
                            destination: destination,
                        });
                        Ok(destination)
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod test {

    use super::*;
    use syntax::{
        Instructions,
        Operator::{Addition, Multiplication},
    };

    #[test]
    fn basic_case() {
        let expr = Expr::Number(5.0);
        let mut program = Program::new();

        let result = program.generate_ir(&expr);

        assert_eq!(result, Ok(0));

        assert_eq!(
            program.instructions,
            vec![Instructions::LoadConstant {
                value: 5.0,
                destination: 0
            }]
        );
    }

    #[test]
    fn simple_addition() {
        let expr = Expr::Binary {
            left: Box::new(Number(5.0)),
            operator: Addition,
            right: Box::new(Number(3.0)),
        };
        let mut program = Program::new();

        let result = program.generate_ir(&expr);

        assert_eq!(result, Ok(2));

        assert_eq!(
            program.instructions,
            vec![
                Instructions::LoadConstant {
                    value: 5.0,
                    destination: 0
                },
                Instructions::LoadConstant {
                    value: 3.0,
                    destination: 1
                },
                Instructions::Add {
                    left: 0,
                    right: 1,
                    destination: 2
                }
            ]
        );
    }

    #[test]
    fn add_and_star() {
        let expr = Expr::Binary {
            left: Box::new(Number(5.0)),
            operator: Addition,
            right: Box::new(Expr::Binary {
                left: Box::new(Number(3.0)),
                operator: Multiplication,
                right: Box::new(Number(2.0)),
            }),
        };

        let mut program = Program::new();

        let result = program.generate_ir(&expr);

        assert!(result.is_ok());

        assert_eq!(
            program.instructions,
            vec![
                Instructions::LoadConstant {
                    value: 5.0,
                    destination: 0
                },
                Instructions::LoadConstant {
                    value: 3.0,
                    destination: 1
                },
                Instructions::LoadConstant {
                    value: 2.0,
                    destination: 2
                },
                Instructions::Multiply {
                    left: 1,
                    right: 2,
                    destination: 3
                },
                Instructions::Add {
                    left: 0,
                    right: 3,
                    destination: 4
                },
            ]
        );
    }
}
