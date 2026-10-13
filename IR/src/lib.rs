use std::collections::HashMap;
use syntax::{
    CalculatorError, Expr::Number, FunctionDefinition, Instructions, SourceProgram, Temp,
};
pub use syntax::{CalculatorError::IrError, Expr, Operator};

#[derive(Debug, PartialEq)]
pub struct Program {
    pub instructions: Vec<Instructions>,
    pub functions: Vec<FunctionIr>,
    next_temp: usize,
    variables: HashMap<String, Temp>,
    function_signatures: HashMap<String, usize>,
}

#[derive(Debug, PartialEq)]
pub struct FunctionIr {
    pub name: String,
    pub instructions: Vec<Instructions>,
    pub result_temp: Temp,
}

impl Program {
    pub fn new() -> Self {
        // Creates program
        Self {
            instructions: Vec::new(),
            functions: Vec::new(),
            next_temp: 0,
            variables: HashMap::new(),
            function_signatures: HashMap::new(),
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
            Expr::Variable(name) => self
                .variables
                .get(name)
                .copied()
                .ok_or_else(|| CalculatorError::UndefinedVariable(name.clone())),
            Expr::Assignment { name, value } => {
                let value_temp = self.generate_ir(value)?;
                self.variables.insert(name.clone(), value_temp);
                Ok(value_temp)
            }
            Expr::FunctionCall { name, arguments } => {
                let expected = self
                    .function_signatures
                    .get(name)
                    .copied()
                    .ok_or_else(|| CalculatorError::UndefinedFunction(name.clone()))?;
                if arguments.len() != expected {
                    return Err(CalculatorError::FunctionArity {
                        name: name.clone(),
                        expected,
                        actual: arguments.len(),
                    });
                }
                let argument_temps = arguments
                    .iter()
                    .map(|argument| self.generate_ir(argument))
                    .collect::<Result<Vec<_>, _>>()?;
                let destination = self.new_temp();
                self.instructions.push(Instructions::Call {
                    name: name.clone(),
                    arguments: argument_temps,
                    destination,
                });
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
                            destination,
                        });
                        Ok(destination)
                    }
                    Operator::Subtraction => {
                        let destination = self.new_temp();
                        self.instructions.push(Instructions::Subtract {
                            left: left_gen,
                            right: right_gen,
                            destination,
                        });
                        Ok(destination)
                    }
                    Operator::Division => {
                        let destination = self.new_temp();
                        self.instructions.push(Instructions::Divide {
                            left: left_gen,
                            right: right_gen,
                            destination,
                        });
                        Ok(destination)
                    }
                    Operator::Multiplication => {
                        let destination = self.new_temp();
                        self.instructions.push(Instructions::Multiply {
                            left: left_gen,
                            right: right_gen,
                            destination,
                        });
                        Ok(destination)
                    }
                }
            }
        }
    }

    pub fn generate_program_ir(&mut self, source: &SourceProgram) -> Result<Temp, CalculatorError> {
        self.instructions.clear();
        self.functions.clear();
        self.variables.clear();
        self.function_signatures.clear();
        self.next_temp = 0;

        for function in &source.functions {
            if function.name == "main" {
                return Err(CalculatorError::ReservedFunctionName(function.name.clone()));
            }
            if self
                .function_signatures
                .insert(function.name.clone(), function.parameters.len())
                .is_some()
            {
                return Err(CalculatorError::DuplicateFunction(function.name.clone()));
            }
        }

        let mut compiled_functions = Vec::with_capacity(source.functions.len());
        for function in &source.functions {
            compiled_functions.push(self.generate_function_ir(function)?);
        }
        self.functions = compiled_functions;

        self.generate_ir(&source.expression)
    }

    fn generate_function_ir(
        &self,
        function: &FunctionDefinition,
    ) -> Result<FunctionIr, CalculatorError> {
        let mut body = Program::new();
        body.function_signatures = self.function_signatures.clone();
        for (index, parameter) in function.parameters.iter().enumerate() {
            let destination = body.new_temp();
            body.instructions
                .push(Instructions::LoadArgument { index, destination });
            body.variables.insert(parameter.clone(), destination);
        }
        let result_temp = body.generate_ir(&function.body)?;
        Ok(FunctionIr {
            name: function.name.clone(),
            instructions: body.instructions,
            result_temp,
        })
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
    fn variable_assignment_reuses_value_temporary() {
        let expr = Expr::Assignment {
            name: "x".into(),
            value: Box::new(Expr::Binary {
                left: Box::new(Number(4.0)),
                operator: Addition,
                right: Box::new(Number(3.0)),
            }),
        };
        let mut program = Program::new();

        assert_eq!(program.generate_ir(&expr), Ok(2));
        assert_eq!(program.generate_ir(&Expr::Variable("x".into())), Ok(2));
    }

    #[test]
    fn generates_nested_function_calls() {
        let source = SourceProgram {
            functions: vec![
                FunctionDefinition {
                    name: "add".into(),
                    parameters: vec!["a".into(), "b".into()],
                    body: Expr::Binary {
                        left: Box::new(Expr::Variable("a".into())),
                        operator: Addition,
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
                arguments: vec![Number(4.0)],
            },
        };
        let mut program = Program::new();

        assert_eq!(program.generate_program_ir(&source), Ok(1));
        assert!(matches!(
            program.instructions.as_slice(),
            [Instructions::LoadConstant { value: 4.0, .. }, Instructions::Call { name, .. }]
                if name == "twice"
        ));
        assert_eq!(program.functions.len(), 2);
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
    #[test]
    fn bracket_testing() {
        let expr = Expr::Binary {
            left: Box::new(Number(5.0)),
            operator: Multiplication,
            right: Box::new(Expr::Binary {
                left: Box::new(Number(3.0)),
                operator: Addition,
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
                Instructions::Add {
                    left: 1,
                    right: 2,
                    destination: 3
                },
                Instructions::Multiply {
                    left: 0,
                    right: 3,
                    destination: 4
                },
            ]
        );
    }
}
