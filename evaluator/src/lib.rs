use std::collections::HashMap;
use syntax::{
    CalculatorError, Expr, FunctionDefinition,
    Operator::{Addition, Division, Multiplication, Subtraction},
    SourceProgram,
};

#[derive(Debug, Default)]
pub struct Evaluator {
    variables: HashMap<String, f64>,
    functions: HashMap<String, FunctionDefinition>,
}

impl Evaluator {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn evaluate(&mut self, expr: Expr) -> Result<f64, CalculatorError> {
        evaluate_expr(&expr, &self.functions, &mut self.variables)
    }

    pub fn evaluate_program(&mut self, program: &SourceProgram) -> Result<f64, CalculatorError> {
        let mut new_names = HashMap::new();
        for function in &program.functions {
            if function.name == "main" {
                return Err(CalculatorError::ReservedFunctionName(function.name.clone()));
            }
            if self.functions.contains_key(&function.name)
                || new_names.insert(function.name.as_str(), ()).is_some()
            {
                return Err(CalculatorError::DuplicateFunction(function.name.clone()));
            }
        }
        for function in &program.functions {
            self.functions
                .insert(function.name.clone(), function.clone());
        }
        evaluate_expr(&program.expression, &self.functions, &mut self.variables)
    }
}

pub fn evaluator(binary: Expr) -> Result<f64, CalculatorError> {
    Evaluator::new().evaluate(binary)
}

fn evaluate_expr(
    binary: &Expr,
    functions: &HashMap<String, FunctionDefinition>,
    variables: &mut HashMap<String, f64>,
) -> Result<f64, CalculatorError> {
    match binary {
        Expr::Binary {
            left,
            operator,
            right,
        } => {
            let left_value = evaluate_expr(left, functions, variables)?;
            let right_value = evaluate_expr(right, functions, variables)?;
            match operator {
                Addition => Ok(left_value + right_value),
                Subtraction => Ok(left_value - right_value),
                Division => {
                    if right_value != 0.0 {
                        Ok(left_value / right_value)
                    } else {
                        Err(CalculatorError::DivisionByZero)
                    }
                }
                Multiplication => Ok(left_value * right_value),
            }
        }
        Expr::Number(value) => Ok(*value),
        Expr::Variable(name) => variables
            .get(name)
            .copied()
            .ok_or_else(|| CalculatorError::UndefinedVariable(name.clone())),
        Expr::Assignment { name, value } => {
            let value = evaluate_expr(value, functions, variables)?;
            variables.insert(name.clone(), value);
            Ok(value)
        }
        Expr::FunctionCall { name, arguments } => {
            let function = functions
                .get(name)
                .cloned()
                .ok_or_else(|| CalculatorError::UndefinedFunction(name.clone()))?;
            if arguments.len() != function.parameters.len() {
                return Err(CalculatorError::FunctionArity {
                    name: name.clone(),
                    expected: function.parameters.len(),
                    actual: arguments.len(),
                });
            }

            let mut local_variables = HashMap::new();
            for (parameter, argument) in function.parameters.iter().zip(arguments) {
                let value = evaluate_expr(argument, functions, variables)?;
                local_variables.insert(parameter.clone(), value);
            }
            evaluate_expr(&function.body, functions, &mut local_variables)
        }
    }
}

#[cfg(test)]
pub mod tests {

    use super::*;
    use syntax::{
        CalculatorError::DivisionByZero,
        Expr::{self, Number},
        Operator,
    };

    #[test]
    fn evaluates_assignment() {
        let assignment = Expr::Assignment {
            name: "x".into(),
            value: Box::new(Expr::Binary {
                left: Box::new(Number(4.0)),
                operator: Addition,
                right: Box::new(Number(3.0)),
            }),
        };

        assert_eq!(evaluator(assignment), Ok(7.0));
    }

    #[test]
    fn evaluates_reference_in_a_later_expression() {
        let mut evaluator = Evaluator::new();

        assert_eq!(
            evaluator.evaluate(Expr::Assignment {
                name: "x".into(),
                value: Box::new(Expr::Binary {
                    left: Box::new(Number(4.0)),
                    operator: Addition,
                    right: Box::new(Number(3.0)),
                }),
            }),
            Ok(7.0)
        );
        assert_eq!(
            evaluator.evaluate(Expr::Binary {
                left: Box::new(Expr::Variable("x".into())),
                operator: Addition,
                right: Box::new(Number(1.0)),
            }),
            Ok(8.0)
        );
    }

    #[test]
    fn evaluates_nested_function_calls() {
        let program = SourceProgram {
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

        assert_eq!(Evaluator::new().evaluate_program(&program), Ok(8.0));
    }

    #[test]
    fn simple_star() {
        let binary = Expr::Binary {
            left: Box::new(Expr::Number(3.0)),
            operator: Operator::Multiplication,
            right: Box::new(Expr::Number(5.0)),
        };

        assert_eq!(evaluator(binary), Ok(15.0));
    }
    #[test]
    fn simple_addition() {
        let binary = Expr::Binary {
            left: Box::new(Number(3.0)),
            operator: Subtraction,
            right: Box::new(Number(5.0)),
        };

        assert_eq!(evaluator(binary), Ok(-2.0))
    }
    #[test]
    fn simple_division() {
        let binary = Expr::Binary {
            left: Box::new(Number(3.0)),
            operator: Division,
            right: Box::new(Number(5.0)),
        };

        assert_eq!(evaluator(binary), Ok(0.6));
    }
    #[test]
    fn star_then_add() {
        let binary = Expr::Binary {
            left: Box::new(Expr::Binary {
                left: Box::new(Number(3.0)),
                operator: Multiplication,
                right: Box::new(Number(5.0)),
            }),
            operator: Addition,
            right: Box::new(Number(2.0)),
        };

        assert_eq!(evaluator(binary), Ok(17.0));
    }
    #[test]
    fn add_then_star() {
        let binary = Expr::Binary {
            left: Box::new(Number(3.0)),
            operator: Addition,
            right: Box::new(Expr::Binary {
                left: Box::new(Number(3.0)),
                operator: Multiplication,
                right: Box::new(Number(5.0)),
            }),
        };

        assert_eq!(evaluator(binary), Ok(18.0));
    }
    #[test]
    fn add_then_star_then_sub() {
        let binary = Expr::Binary {
            left: Box::new(Expr::Binary {
                left: Box::new(Number(3.0)),
                operator: Addition,
                right: Box::new(Expr::Binary {
                    left: Box::new(Number(5.0)),
                    operator: Multiplication,
                    right: Box::new(Number(2.0)),
                }),
            }),
            operator: Subtraction,
            right: Box::new(Number(4.0)),
        };

        assert_eq!(evaluator(binary), Ok(9.0));
    }

    #[test]
    fn division_by_zero() {
        let binary = Expr::Binary {
            left: Box::new(Number(2.0)),
            operator: Division,
            right: Box::new(Number(0.0)),
        };

        assert_eq!(evaluator(binary), Err(DivisionByZero));
    }
}
